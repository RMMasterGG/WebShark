use std::sync::Arc;
use bytes::Bytes;
use tokio::io::DuplexStream;
use crate::{Request};
use crate::routing::socket_handler::SocketHandler;


pub struct Socket {
    path: &'static str,
    handler: Arc<dyn SocketHandler<(Request<Bytes>, DuplexStream)> + Sync + Send + 'static>
}

impl Default for Socket {
    fn default() -> Self {
        Self {
            path: "",
            handler: Arc::new(|_request: Request<Bytes>, _conn: DuplexStream|{}),
        }
    }
}

impl Socket {
    pub fn new<H, Args>(path: &'static str, handler: H) -> Self
    where H: SocketHandler<Args> + Sync + Send + 'static
    {
        let wrapper = move |request: Request<Bytes>, conn: DuplexStream| handler.invoke(request, conn);

        Self {
            path,
            handler: Arc::new(wrapper),
        }

    }

    pub fn path(&self) -> &'static str { self.path }
    pub fn handler(&self) -> Arc<dyn SocketHandler<(Request<Bytes>, DuplexStream)>> {
        self.handler.clone()
    }
}