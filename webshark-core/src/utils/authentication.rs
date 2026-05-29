//! Фаел аунтификации, должнон работать как в спринге секурити,
//! к нему фаел для удобства при работе с параметрами функции
//!

use crate::{Request, Response};
use base64::{Engine, prelude::BASE64_STANDARD};
use std::sync::Arc;
use bytes::Bytes;

#[derive(Debug, Default, Clone)]
pub struct Authentication {
    credentials: Authorization,
    user_details: Option<UserDetails>,
    is_authenticated: bool
}

impl Authentication {
    pub fn new(credentials: Authorization, user_details: Option<UserDetails>, is_authenticated: bool) -> Self {
        Self {
            credentials,
            user_details,
            is_authenticated,
        }
    }
    
    pub fn credentials(&self) -> &Authorization { &self.credentials }
}

#[derive(Debug, Clone, Default)]
pub enum Authorization {
    Bearer(String),
    Basic(String, String),
    #[default]
    None,    // Если заголовок не передан
    Invalid,               // Если заголовок есть, но формат сломан
}

impl Authorization {
    pub fn parse(auth_str: impl AsRef<str>) -> Self {
        let auth_str = auth_str.as_ref().trim();

        if auth_str.is_empty() {
            return Self::None;
        }

        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            let clean_token = token.trim();
            if clean_token.is_empty() {
                Self::Invalid
            } else {
                Self::Bearer(clean_token.into())
            }
        } else if let Some(base64_data) = auth_str.strip_prefix("Basic ") {
            BASE64_STANDARD
                .decode(base64_data.trim())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .and_then(|decoded_str| {
                    decoded_str
                        .split_once(':')
                        .map(|(l, p)| Self::Basic(l.trim().to_string(), p.trim().to_string()))
                })
                .unwrap_or(Self::Invalid)
        } else {
            Self::Invalid
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserDetails {
    username: String,
    password: String,
    authorities: Vec<String>,
}

impl UserDetails {
    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }

    pub fn authorities(&self) -> Vec<String> {
        self.authorities.clone()
    }
}

#[derive(Default, Clone)]
pub struct FilterChain {
    filters: Vec<Arc<dyn Filter>>,
}

impl FilterChain {
    pub fn new(filters: Vec<Arc<dyn Filter>>) -> Self {
        Self { filters }
    }

    pub fn execute(
        &mut self,
        request: Request<Bytes>,
        handler: impl Fn(Request<Bytes>) -> Result<Response<Bytes>, &'static str>,
    ) -> Result<Response<Bytes>, &'static str> {
        let mut context = FilterContext::new(&self.filters);
        context.next_filter(request, &handler)
    }
}

pub struct FilterContext<'a> {
    filters: &'a [Arc<dyn Filter>],
    current_index: usize,
}

impl<'a> FilterContext<'a> {
    fn new(filters: &'a [Arc<dyn Filter>]) -> Self {
        Self {
            filters,
            current_index: 0,
        }
    }

    pub fn next_filter(
        &mut self,
        request: Request<Bytes>,
        handler: &dyn Fn(Request<Bytes>) -> Result<Response<Bytes>, &'static str>
    ) -> Result<Response<Bytes>, &'static str> {
        if self.current_index < self.filters.len() {
            let index = self.current_index;
            self.current_index += 1;

            let filter = &self.filters[index];
            filter.do_filter(request, self, handler)
        } else {
            handler(request)
        }
    }
}

pub trait Filter: Send + Sync {
    fn do_filter(
        &self,
        request: Request<Bytes>,
        context: &mut FilterContext,
        handler: &dyn Fn(Request<Bytes>) -> Result<Response<Bytes>, &'static str>,
    ) -> Result<Response<Bytes>, &'static str>;
}
