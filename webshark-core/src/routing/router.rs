//! Модуль, представляющий маршрутизатор (Router) сервера.
//!
//! Хранит в себе карту соответствий строковых ключей (Метод + Путь)
//! и стёртых по типу функций-обработчиков (Handler).

use crate::auth::authentication::{Filter, FilterChain};
use crate::routing::route_handler::{RouteHandler};
use crate::routing::route::Route;
use crate::routing::scope::Scope;
use std::collections::HashMap;
use std::sync::Arc;
use bytes::Bytes;
use http::Method;
use crate::{Request, Response};
use crate::routing::socket_context::WebSocketContext;
use crate::routing::socket_handler::SocketHandler;
use crate::utils::other::BoxFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteType {
    Http,
    WebSocket,
    WebTransport,
}

pub type HttpHandler = Arc<dyn RouteHandler<Request<Bytes>> + Send + Sync + 'static>;
pub type WSHandler = Arc<dyn SocketHandler<(Request<Bytes>, WebSocketContext)> + Send + Sync + 'static>;
pub type WTHandler = Arc<dyn SocketHandler<(Request<Bytes>, WebSocketContext)> + Send + Sync + 'static>;

#[derive(Clone)]
pub enum Handler {
    Http(HttpHandler),
    WebSocket(WSHandler),
    WebTransport(WTHandler),
}

impl Handler {
    pub fn invoke_http(&self, req: Request<Bytes>) -> BoxFuture<'static, Response<Bytes>> {
        match self {
            Handler::Http(handler) => handler.invoke(req),
            _ => panic!("Вызов invoke_http для сокет-соединения недопустим!"),
        }
    }
}

#[derive(Clone)]
pub struct CompiledRoute {
    handler: Handler,
    filter_chain: FilterChain,
    route_type: RouteType,
    method: Option<Method>,
}

impl CompiledRoute {
    pub fn filter_chain(&self) -> &FilterChain {
        &self.filter_chain
    }

    pub fn handler(&self) -> &Handler {
        &self.handler
    }

    pub fn is_websocket(&self) -> bool {
        true
    }

    pub fn route_type(&self) -> RouteType {
        self.route_type
    }

    pub fn method(&self) -> Option<Method> {
        self.method.clone()
    }
}

/// Структура маршрутизатора.
///
/// Управляет регистрацией эндпоинтов и поиском нужного обработчика
/// при входящих запросах.
#[derive(Default, Clone)]
pub struct Router {
    routes: HashMap<String, Vec<CompiledRoute>>,
}

impl Router {
    /// Создает новый пустой маршрутизатор.
    pub fn new() -> Self {
        Self::default()
    }

    /// Регистрирует новый маршрут в системе.
    ///
    /// Принимает структуру [`Route`], извлекает из неё HTTP-метод,
    /// путь и упакованный обработчик, после чего сохраняет их в `HashMap`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use webshark::routing::router::Router;
    /// use webshark::routing::route::Route;
    /// use webshark::routing::request::Method;
    /// use webshark::routing::response::Response;
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::new(Method::GET, "/", || Response::ok()));
    /// ```
    #[deprecated]
    pub fn add_route(&mut self, route: Route) {
        let compiled = CompiledRoute {
            handler: Handler::Http(route.handler()),
            filter_chain: FilterChain::default(),
            route_type: RouteType::Http,
            method: Some(route.method()),
        };
        self.insert_route(route.path().to_string(), compiled);
    }

    fn insert_route(&mut self, path: String, route: CompiledRoute) {

        self.routes
            .entry(path)
            .or_default()
            .push(route);
    }

    pub fn add_scope(&mut self, scope: Scope) {
        let mut filter_stack = Vec::new();
        self.compile_scope("", &mut filter_stack, scope);
    }

    fn compile_scope(
        &mut self,
        parent_prefix: &str,
        filter_stack: &mut Vec<Arc<dyn Filter>>,
        mut scope: Scope,
    ) {
        let mut current_prefix = String::with_capacity(parent_prefix.len() + scope.prefix().len());
        current_prefix.push_str(parent_prefix);
        current_prefix.push_str(scope.prefix());
        let previous_filters_count = filter_stack.len();

        filter_stack.extend(scope.filters());

        self.push_endpoints(
            &current_prefix,
            filter_stack,
            RouteType::Http,
            scope.routes().into_iter().map(|r| (r.path(), Some(r.method()), Handler::Http(r.handler())))
        );

        self.push_endpoints(
            &current_prefix,
            filter_stack,
            RouteType::WebSocket,
            scope.sockets().into_iter().map(|s| (s.path(), None, Handler::WebSocket(s.handler())))
        );

        self.push_endpoints(
            &current_prefix,
            filter_stack,
            RouteType::WebTransport,
            scope.transports().into_iter().map(|t| (t.path(), None, Handler::WebTransport(t.handler())))
        );

        for nested in scope.scopes() {
            self.compile_scope(&current_prefix, filter_stack, nested);
        }

        filter_stack.truncate(previous_filters_count);
    }

    fn push_endpoints<I>(&mut self, prefix: &str, filter_stack: &mut Vec<Arc<dyn Filter>>, route_type: RouteType, endpoints: I)
    where
        I: IntoIterator<Item = (&'static str, Option<Method>, Handler)>,
    {

        for (r_path, method, handler) in endpoints {
            let mut full_path = String::with_capacity(prefix.len() + r_path.len());
            full_path.push_str(prefix);
            full_path.push_str(r_path);

            let filter_chain = FilterChain::new(filter_stack.clone());

            self.insert_route(
                full_path,
                CompiledRoute {
                    handler,
                    filter_chain,
                    route_type,
                    method,
                }
            );
        }

    }

    /// Ищет зарегистрированный обработчик по HTTP-методу и текстовому пути.
    ///
    /// Возвращает `Some(&dyn Handler<Request>)`, если маршрут найден,
    /// или `None`, если такого эндпоинта не существует.
    pub fn get_handler(
        &self,
        method: &Method,
        route_type: RouteType,
        path: &str,
    ) -> Option<Handler> {

        let compiled_route = self.get_route(method, route_type, path)?;

        Some(compiled_route.handler.clone())
    }

    pub fn get_route(&self, method: &Method, route_type: RouteType, path: &str) -> Option<&CompiledRoute> {

        let routes_list = self.routes.get(path)?;

        for compiled in routes_list {
            match route_type {
                RouteType::Http => {
                    if compiled.route_type == RouteType::Http && compiled.method == Some(method.clone()) {
                        return Some(compiled)
                    }
                }
                _ => {
                    if compiled.route_type == route_type {
                        return Some(compiled)
                    }
                }
            }
        }

        None
    }
}

// TODO Асинхронные тесты

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::Response;
//
//     fn mock_home_handler() -> Response<Bytes> {
//         Response::ok_body("home")
//     }
//
//     fn mock_post_handler() -> Response<Bytes> {
//         Response::created_body("created")
//     }
//
//     mod router_lifecycle {
//         use super::*;
//
//         #[test]
//         fn test_new_router_is_empty() {
//             let router = Router::new();
//
//             assert!(router.get_handler(&Method::GET, RouteType::Http, "/").is_none());
//         }
//
//         #[test]
//         fn test_add_and_get_route_success() {
//             let mut router = Router::new();
//
//             router.add_route(Route::new(Method::GET, "/tests", mock_home_handler));
//
//             let handler_opt = router.get_handler(&Method::GET, RouteType::Http, "/tests");
//             assert!(handler_opt.is_some());
//
//             let handler = handler_opt.unwrap();
//             let response = handler.invoke_http(Request::default());
//             assert_eq!("home", response.body_as_str());
//         }
//
//         #[test]
//         fn test_route_not_found() {
//             let mut router = Router::new();
//             router.add_route(Route::new(Method::GET, "/", mock_home_handler));
//
//             assert!(router.get_handler(&Method::GET, RouteType::Http, "/profile").is_none());
//
//             assert!(router.get_handler(&Method::POST, RouteType::Http, "/").is_none());
//         }
//
//         #[test]
//         fn test_overwrite_existing_route() -> Result<(), Box<dyn std::error::Error>> {
//             let mut router = Router::new();
//
//             router.add_route(Route::new(Method::POST, "/submit", mock_home_handler));
//             router.add_route(Route::new(Method::POST, "/submit", mock_post_handler));
//
//             let handler = router
//                 .get_handler(&Method::POST, RouteType::Http, "/submit")
//                 .ok_or("Маршрут не найден")?;
//             let response = handler.invoke_http(Request::default());
//
//             assert_eq!("created", response.body_as_str());
//             Ok(())
//         }
//     }
// }
