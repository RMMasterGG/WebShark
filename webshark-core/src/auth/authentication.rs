//! Фаел аунтификации, должнон работать как в спринге секурити,
//! к нему фаел для удобства при работе с параметрами функции
//!

use crate::utils::other::BoxFuture;
use crate::{Request, Response};
use base64::{Engine, prelude::BASE64_STANDARD};
use bytes::Bytes;
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct Authentication {
    credentials: Authorization,
    user_details: Option<UserDetails>,
    is_authenticated: bool,
}

impl Authentication {
    pub fn new(
        credentials: Authorization,
        user_details: Option<UserDetails>,
        is_authenticated: bool,
    ) -> Self {
        Self {
            credentials,
            user_details,
            is_authenticated,
        }
    }

    pub fn credentials(&self) -> &Authorization {
        &self.credentials
    }
}

#[derive(Debug, Clone, Default)]
pub enum Authorization {
    Bearer(String),
    Basic(String, String),
    #[default]
    None, // Если заголовок не передан
    Invalid, // Если заголовок есть, но формат сломан
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

    pub async fn execute<F, Fut>(
        &mut self,
        request: Request<Bytes>,
        handler: F,
    ) -> Result<Response<Bytes>, &'static str>
    where
        F: Fn(Request<Bytes>) -> BoxFuture<'static, Result<Response<Bytes>, &'static str>>
            + Send
            + Sync
            + 'static,
    {
        let mut context = FilterContext::new(self.filters.clone(), Box::new(handler));
        context.next_filter(request).await
    }
}

pub struct FilterContext {
    filters: Vec<Arc<dyn Filter>>,
    current_index: usize,
    handler: Box<
        dyn Fn(Request<Bytes>) -> BoxFuture<'static, Result<Response<Bytes>, &'static str>>
            + Send
            + Sync,
    >,
}

impl FilterContext {
    fn new(
        filters: Vec<Arc<dyn Filter>>,
        handler: Box<
            dyn Fn(Request<Bytes>) -> BoxFuture<'static, Result<Response<Bytes>, &'static str>>
                + Send
                + Sync,
        >,
    ) -> Self {
        Self {
            filters,
            current_index: 0,
            handler,
        }
    }

    pub fn next_filter(
        &mut self,
        request: Request<Bytes>,
    ) -> BoxFuture<'_, Result<Response<Bytes>, &'static str>> {
        if self.current_index < self.filters.len() {
            let index = self.current_index;
            self.current_index += 1;

            let filter = self.filters[index].clone();

            Box::pin(async move {
                filter.do_filter(request, self).await
            })
        } else {
            Box::pin(async move {
                (self.handler)(request).await
            })
        }
    }
}

pub trait Filter: Send + Sync + 'static {
    fn do_filter<'a>(
        &self,
        request: Request<Bytes>,
        context: &'a mut FilterContext,
    ) -> BoxFuture<'a, Result<Response<Bytes>, &'static str>>;
}
