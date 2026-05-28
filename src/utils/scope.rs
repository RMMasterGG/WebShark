use std::sync::Arc;
use crate::{Route};
use crate::utils::authentication::Filter;
use crate::utils::socket::Socket;

#[derive(Default)]
pub struct Scope {
    prefix: &'static str,
    filters: Vec<Arc<dyn Filter>>,
    routes: Vec<Route>,
    sockets: Vec<Socket>,
    nested_scopes: Vec<Scope>,
}

impl Scope {
    pub fn new(prefix: &'static str) -> Self {
        Self { prefix, ..Default::default() }
    }

    pub fn with_filter(mut self, filter: impl Filter + 'static) -> Self {
        self.filters.push(Arc::new(filter));
        self
    }

    pub fn add_route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }

    pub fn add_websocket(mut self, socket: Socket) -> Self {
        self.sockets.push(socket);
        self
    }

    pub fn add_webtransport(mut self, socket: Socket) -> Self {
        self.sockets.push(socket);
        self
    }

    pub fn nest(mut self, scope: Scope) -> Self {
        self.nested_scopes.push(scope);
        self
    }

    pub fn prefix(&self) -> &'static str {
        self.prefix
    }

    pub fn routes(&mut self) -> Vec<Route> {
        std::mem::take(&mut self.routes)
    }

    pub fn sockets(&mut self) -> Vec<Socket> {
        std::mem::take(&mut self.sockets)
    }

    pub fn scopes(&mut self) -> Vec<Scope> {
        std::mem::take(&mut self.nested_scopes)
    }

    pub fn filters(&mut self) -> Vec<Arc<dyn Filter>> {
        std::mem::take(&mut self.filters)
    }
}