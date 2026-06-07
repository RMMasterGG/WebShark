use std::marker::PhantomData;
use std::pin::Pin;

pub(crate) struct BoxedHandler<H, Args> {
    pub inner: H,
    pub _marker: PhantomData<Args>,
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;