use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use tokio::sync::mpsc::UnboundedSender;
use serde_json::Value;
use crate::context::{SenderContext};

pub type Json = Value;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type ListenerFunction<TRequest> = Arc<dyn Fn(UnboundedSender<TRequest>) -> BoxFuture<()> + Send + Sync>;
pub type ListenerFunctionWrapper = Arc<dyn Fn() -> BoxFuture<()> + Send + Sync>;
pub type HandlerFunction<TRequest> = Arc<dyn Fn(TRequest) -> BoxFuture<()> + Send + Sync>;
pub type MessageHandlerFunction<TContext> = Arc<dyn Fn(TContext) -> BoxFuture<()> + Send + Sync>;
pub type SenderFunction = Arc<dyn Fn(SenderContext) -> BoxFuture<()> + Send + Sync>;
// pub type CallNextMiddleware = Arc<dyn Fn(MiddlewareContext) -> BoxFuture<()> + Send + Sync + 'static>;
// pub type Middleware = Arc<dyn Fn(MiddlewareContext, CallNextMiddleware) -> BoxFuture<()> + Send + Sync + 'static>;