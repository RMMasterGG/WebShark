//! Модуль, отвечающий за абстракцию и затирание типов функций-обработчиков (эндпоинтов).
//!
//! Позволяет роутеру регистрировать и вызывать как функции без аргументов,
//! так и функции, принимающие объект [`Request`].

use bytes::Bytes;
use crate::{Request, Response};
use crate::utils::other::{BoxFuture, BoxedHandler};


/// Трейт, инкапсулирующий вызов эндпоинта.
///
/// Использует маркер обобщения `Args` для разделения функций с разной сигнатурой
/// на этапе компиляции, предотвращая конфликт реализаций.
pub trait RouteHandler<Args> {
    fn invoke(&self, req: Request<Bytes>) -> BoxFuture<'static, Response<Bytes>>;
}

/// Реализация для функций и замыканий, не принимающих аргументов.
impl<F, Fut> RouteHandler<()> for F
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<Bytes>> + Send + 'static,
{
    fn invoke(&self, _req: Request<Bytes>) -> BoxFuture<'static, Response<Bytes>> {
        let fut = self();
        Box::pin(fut)
    }
}

/// Реализация для функций и замыканий, принимающих [`Request`] по значению.
impl<F, Fut> RouteHandler<Request<Bytes>> for F
where
    F: Fn(Request<Bytes>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<Bytes>> + Send + 'static,
{
    fn invoke(&self, req: Request<Bytes>) -> BoxFuture<'static, Response<Bytes>> {
        let fut = self(req);
        Box::pin(fut)
    }
}

impl<H, Args> RouteHandler<(Request<Bytes>)> for BoxedHandler<H, Args>
where
    H: RouteHandler<Args> + Sync + Send + 'static,
    Args: Send + Sync + 'static,
{
    fn invoke(
        &self,
        req: Request<Bytes>
    ) -> BoxFuture<'static, Response<Bytes>> {
        self.inner.invoke(req)
    }
}

// #[cfg(test)]
// mod tests {
//     use http::Method;
//     use super::*;
//
//     fn mock_index_handler() -> Response<Bytes> {
//         Response::ok_body("index_page")
//     }
//
//     fn mock_api_handler(req: Request<Bytes>) -> Response<Bytes> {
//         if req.method() == Method::POST {
//             Response::created_body("success")
//         } else {
//             Response::bad_request()
//         }
//     }
//
//     mod handler_trait_polymorphism {
//         use super::*;
//
//         #[test]
//         fn test_call_handler_without_arguments() {
//             let req = Request::default();
//             let response = mock_index_handler.invoke(req);
//
//             assert_eq!("index_page", response.body_as_str());
//         }
//
//         #[test]
//         fn test_call_handler_with_request_argument() -> Result<(), Box<dyn std::error::Error>> {
//             let mut raw_request = "POST /api HTTP/1.1\r\nContent-Length: 0\r\n\r\n".as_bytes();
//             let req = Request::parse(&mut raw_request)?;
//
//             let response = mock_api_handler.invoke(req);
//
//             assert_eq!("success", response.body_as_str());
//             Ok(())
//         }
//
//         #[test]
//         fn test_call_anonymous_closures() -> Result<(), Box<dyn std::error::Error>> {
//             let closure_no_args = || Response::ok_body("closure_1");
//             let res_1 = closure_no_args.invoke(Request::default());
//             assert_eq!("closure_1", res_1.body_as_str());
//
//             let closure_with_args = |req: Request<Bytes>| Response::ok_body(format!("path: {}", req.uri().path()));
//
//             let mut raw_request = "GET /test_path HTTP/1.1\r\nContent-Length: 0\r\n\r\n".as_bytes();
//             let req = Request::parse(&mut raw_request)?;
//
//             let res_2 = closure_with_args.invoke(req);
//             assert_eq!("path: /test_path", res_2.body_as_str());
//             Ok(())
//         }
//     }
// }
