use std::pin::Pin;
use std::future::Future;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;