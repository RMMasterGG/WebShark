//! Модуль, отвечающий за абстракцию и затирание типов функций-обработчиков (эндпоинтов).
//!
//! Позволяет роутеру регистрировать и вызывать как функции без аргументов,
//! так и функции, принимающие объект [`Request`].

use bytes::Bytes;
use tokio::io::DuplexStream;
use crate::utils::request::Request;

/// Трейт, инкапсулирующий вызов эндпоинта.
///
/// Использует маркер обобщения `Args` для разделения функций с разной сигнатурой
/// на этапе компиляции, предотвращая конфликт реализаций.
pub trait SocketHandler<Args> {
    fn invoke(&self, req: Request<Bytes>, conn: DuplexStream);
}

impl<F> SocketHandler<()> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn invoke(&self, _req: Request<Bytes>, _conn: DuplexStream) {
        self()
    }
}

/// Реализация для функций и замыканий, не принимающих аргументов.
impl<F> SocketHandler<DuplexStream> for F
where
    F: Fn(DuplexStream) + Send + Sync + 'static,
{
    fn invoke(&self, _req: Request<Bytes>, conn: DuplexStream) {
        self(conn)
    }
}

/// Для функций fn(req, conn)
impl<F> SocketHandler<(Request<Bytes>, DuplexStream)> for F
where
    F: Fn(Request<Bytes>, DuplexStream) + Send + Sync + 'static,
{
    fn invoke(&self, req: Request<Bytes>, conn: DuplexStream) {
        self(req, conn)
    }
}

#[cfg(test)]
mod tests {
    use http::Method;
    use super::*;

    mod handler_trait_polymorphism {
        use super::*;


    }
}

