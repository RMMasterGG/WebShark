//! Модуль, описывающий структуру конкретного маршрута (Route).
//!
//! Объединяет в себе HTTP-метод, текстовый путь и стёртый по типу
//! обработчик (Handler), превращая их в единую сущность для роутера.

use std::sync::Arc;
use http::Method;
use bytes::Bytes;
use crate::utils::route_handler::RouteHandler;
use crate::utils::request::{Request};
use crate::utils::response::Response;

/// Структура, представляющая зарегистрированный эндпоинт.
pub struct Route {
    method: Method,
    path: &'static str,
    handler: Arc<dyn RouteHandler<Request<Bytes>> + Sync + Send + 'static>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            method: Default::default(),
            path: "",
            handler: Arc::new(|_request: Request<Bytes>| Response::default()),
        }
    }
}

impl Route {
    /// Создает новый маршрут, автоматически затирая тип аргументов функции-обработчика.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use webshark::utils::request::{Method, Request};
    /// use webshark::utils::response::Response;
    /// use webshark::utils::route::Route;
    ///
    /// // Функция без аргументов
    /// fn index() -> Response { Response::ok() }
    /// let route_home = Route::new(Method::GET, "/", index);
    ///
    /// // Функция с аргументом Request
    /// fn api(req: Request) -> Response { Response::ok() }
    /// let route_api = Route::new(Method::POST, "/api", api);
    /// ```
    pub fn new<H, Args>(method: Method, path: &'static str, handler: H) -> Self
    where
        H: RouteHandler<Args> + Sync + Send + 'static,
    {
        let wrapper = move |request: Request<Bytes>| handler.invoke(request);

        Self {
            method,
            path,
            handler: Arc::new(wrapper),
        }
    }

    /// Уничтожает структуру `Route` и возвращает владение упакованным обработчиком.
    ///
    /// Используется роутером при добавлении маршрута в `HashMap`.
    pub fn handler(&self) -> Arc<dyn RouteHandler<Request<Bytes>> + Sync + Send + 'static> {
        self.handler.clone()
    }

    /// Возвращает HTTP-метод маршрута.
    pub fn method(&self) -> Method {
        self.method.clone()
    }

    /// Возвращает текстовый путь маршрута.
    pub fn path(&self) -> &'static str {
        self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_handler() -> Response<Bytes> {
        Response::ok_body("route_test")
    }

    mod route_lifecycle {
        use super::*;

        #[test]
        fn test_route_default_state() {
            let default_route = Route::default();

            assert_eq!(Method::default(), default_route.method());
            assert_eq!("", default_route.path());

            let handler = default_route.handler();
            let response = handler.invoke(Request::default());

            assert_eq!(format!("{}", Response::default()), format!("{}", response));
        }

        #[test]
        fn test_route_new_initialization() {
            let route = Route::new(Method::POST, "/submit", mock_handler);

            assert_eq!(Method::POST, route.method());
            assert_eq!("/submit", route.path());

            let handler = route.handler();
            let response = handler.invoke(Request::default());
            assert_eq!("route_test", response.body_as_str());
        }
    }

}