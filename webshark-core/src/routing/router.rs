//! Модуль, представляющий маршрутизатор (Router) сервера.
//!
//! Хранит в себе карту соответствий строковых ключей (Метод + Путь)
//! и стёртых по типу функций-обработчиков (Handler).

use crate::auth::authentication::{Filter, FilterChain};
use crate::routing::route_handler::RouteHandler;
use crate::routing::route::Route;
use crate::routing::scope::Scope;
use std::collections::HashMap;
use std::sync::Arc;
use bytes::Bytes;
use http::Method;
use crate::Request;

#[derive(Clone)]
pub struct CompiledRoute {
    handler: Arc<dyn RouteHandler<Request<Bytes>> + Send + Sync>,
    filter_chain: FilterChain,
}

impl CompiledRoute {
    pub fn filter_chain(&self) -> &FilterChain {
        &self.filter_chain
    }

    pub fn handler(&self) -> &Arc<dyn RouteHandler<Request<Bytes>> + Send + Sync> {
        &self.handler
    }
}

/// Структура маршрутизатора.
///
/// Управляет регистрацией эндпоинтов и поиском нужного обработчика
/// при входящих запросах.
#[derive(Default, Clone)]
pub struct Router {
    routes: HashMap<Method, HashMap<String, CompiledRoute>>,
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
            handler: route.handler(),
            filter_chain: FilterChain::default(),
        };
        self.insert_route(route.method().clone(), route.path().to_string(), compiled);
    }

    fn insert_route(&mut self, method: Method, path: String, route: CompiledRoute) {
        self.routes
            .entry(method)
            .or_default()
            .insert(path, route);
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

        for route in scope.routes() {
            let mut full_path = String::with_capacity(current_prefix.len() + route.path().len());
            full_path.push_str(current_prefix.as_str());
            full_path.push_str(route.path());
            let chain = FilterChain::new(filter_stack.clone());

            self.insert_route(
                route.method().clone(),
                full_path,
                CompiledRoute {
                    handler: route.handler(),
                    filter_chain: chain,
                }
            );
        }

        for nested in scope.scopes() {
            self.compile_scope(&current_prefix, filter_stack, nested);
        }

        filter_stack.truncate(previous_filters_count);
    }

    /// Ищет зарегистрированный обработчик по HTTP-методу и текстовому пути.
    ///
    /// Возвращает `Some(&dyn Handler<Request>)`, если маршрут найден,
    /// или `None`, если такого эндпоинта не существует.
    pub fn get_handler(
        &self,
        method: &Method,
        path: &str,
    ) -> Option<&(dyn RouteHandler<Request<Bytes>> + Send + Sync + 'static)> {

        let method_map = self.routes.get(method)?;

        let compiled_route = method_map.get(path)?;

        Some(compiled_route.handler.as_ref())
    }

    pub fn get_route(&self, method: &Method, path: &str) -> Option<&CompiledRoute> {
        self.routes.get(method)?.get(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;

    fn mock_home_handler() -> Response<Bytes> {
        Response::ok_body("home")
    }

    fn mock_post_handler() -> Response<Bytes> {
        Response::created_body("created")
    }

    mod router_lifecycle {
        use super::*;

        #[test]
        fn test_new_router_is_empty() {
            let router = Router::new();

            assert!(router.get_handler(&Method::GET, "/").is_none());
        }

        #[test]
        fn test_add_and_get_route_success() {
            let mut router = Router::new();

            router.add_route(Route::new(Method::GET, "/tests", mock_home_handler));

            let handler_opt = router.get_handler(&Method::GET, "/tests");
            assert!(handler_opt.is_some());

            let handler = handler_opt.unwrap();
            let response = handler.invoke(Request::default());
            assert_eq!("home", response.body_as_str());
        }

        #[test]
        fn test_route_not_found() {
            let mut router = Router::new();
            router.add_route(Route::new(Method::GET, "/", mock_home_handler));

            assert!(router.get_handler(&Method::GET, "/profile").is_none());

            assert!(router.get_handler(&Method::POST, "/").is_none());
        }

        #[test]
        fn test_overwrite_existing_route() -> Result<(), Box<dyn std::error::Error>> {
            let mut router = Router::new();

            router.add_route(Route::new(Method::POST, "/submit", mock_home_handler));
            router.add_route(Route::new(Method::POST, "/submit", mock_post_handler));

            let handler = router
                .get_handler(&Method::POST, "/submit")
                .ok_or("Маршрут не найден")?;
            let response = handler.invoke(Request::default());

            assert_eq!("created", response.body_as_str());
            Ok(())
        }
    }
}
