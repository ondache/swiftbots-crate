use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use tokio::sync::mpsc;
use crate::context::{Context, Request, HandlerBackContext, MiddlewareContext};

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type ListenerFunction = Arc<dyn Fn(mpsc::Sender<Request>) -> BoxFuture<()> + Send + Sync>;
pub type HandlerFunction = Arc<dyn Fn(Context) -> BoxFuture<()> + Send + Sync>;
pub type CallNextMiddleware = Arc<dyn Fn(MiddlewareContext) -> BoxFuture<HandlerBackContext> + Send + Sync + 'static>;
pub type Middleware = Arc<dyn Fn(MiddlewareContext, CallNextMiddleware) -> BoxFuture<HandlerBackContext> + Send + Sync + 'static>;