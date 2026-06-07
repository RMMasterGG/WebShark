//! Модуль, отвечающий за абстракцию и затирание типов функций-обработчиков (эндпоинтов).
//!
//! Позволяет роутеру регистрировать и вызывать как функции без аргументов,
//! так и функции, принимающие объект [`Request`].

use crate::Request;
use crate::routing::socket_context::WebSocketContext;
use bytes::Bytes;
use crate::utils::other::{BoxFuture, BoxedHandler};

/// Трейт, инкапсулирующий вызов эндпоинта.
///
/// Использует маркер обобщения `Args` для разделения функций с разной сигнатурой
/// на этапе компиляции, предотвращая конфликт реализаций.
pub trait SocketHandler<Args> {
    fn invoke(&self, req: Request<Bytes>, ctx: WebSocketContext) -> BoxFuture<'static, ()>;
}

impl<F, Fut> SocketHandler<()> for F
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    fn invoke(&self, _req: Request<Bytes>, _ctx: WebSocketContext) -> BoxFuture<'static, ()> {
        let fut = self();
        Box::pin(async move { fut.await })
    }
}

impl<F, Fut> SocketHandler<Request<Bytes>> for F
where
    F: Fn(Request<Bytes>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    fn invoke(&self, req: Request<Bytes>, _ctx: WebSocketContext) -> BoxFuture<'static, ()> {
        let fut = self(req);
        Box::pin(async move { fut.await })
    }
}

impl<F, Fut> SocketHandler<WebSocketContext> for F
where
    F: Fn(WebSocketContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    fn invoke(&self, _req: Request<Bytes>, ctx: WebSocketContext) -> BoxFuture<'static, ()> {
        let fut = self(ctx);
        Box::pin(async move { fut.await })
    }
}

impl<H, Args> SocketHandler<(Request<Bytes>, WebSocketContext)> for BoxedHandler<H, Args>
where
    H: SocketHandler<Args> + Sync + Send + 'static,
    Args: Send + Sync + 'static,
{
    fn invoke(
        &self,
        req: Request<Bytes>,
        ctx: WebSocketContext,
    ) -> BoxFuture<'static, ()> {
        self.inner.invoke(req, ctx)
    }
}

/// Для функций fn(req, conn)
impl<F, Fut> SocketHandler<(Request<Bytes>, WebSocketContext)> for F
where
    F: Fn(Request<Bytes>, WebSocketContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=()> + Send + 'static,
{
    fn invoke(&self, req: Request<Bytes>, ctx: WebSocketContext) -> BoxFuture<'static, ()> {
        let fut = self(req, ctx);
        Box::pin(async move { fut.await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Method;

    mod handler_trait_polymorphism {
        use super::*;
    }
}
