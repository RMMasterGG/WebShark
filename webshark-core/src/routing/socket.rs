use std::marker::PhantomData;
use crate::Request;
use crate::routing::socket_context::WebSocketContext;
use crate::routing::socket_handler::{SocketHandler};
use bytes::Bytes;
use std::sync::Arc;
use crate::utils::other::BoxedHandler;

pub struct Socket {
    path: &'static str,
    handler: Arc<dyn SocketHandler<(Request<Bytes>, WebSocketContext)> + Sync + Send + 'static>,
}

async fn noop_handler(_req: Request<Bytes>, _ctx: WebSocketContext) {}

impl Default for Socket {
    fn default() -> Self {
        Socket::new("", noop_handler)
    }
}

impl Socket {
    pub fn new<H, Args>(path: &'static str, handler: H) -> Self
    where
        H: SocketHandler<Args> + Sync + Send + 'static,
        Args: Send + Sync + 'static,
    {
        let boxed = BoxedHandler {
            inner: handler,
            _marker: PhantomData,
        };

        Self {
            path,
            handler: Arc::new(boxed),
        }
    }

    pub fn path(&self) -> &'static str {
        self.path
    }
    pub fn handler(
        &self,
    ) -> Arc<dyn SocketHandler<(Request<Bytes>, WebSocketContext)> + Sync + Send + 'static> {
        self.handler.clone()
    }
}
