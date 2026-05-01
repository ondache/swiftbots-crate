use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use tokio::sync::mpsc::UnboundedSender;
use crate::context::{SenderContext};

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type ListenerFunction<TRequest> = Arc<dyn Fn(UnboundedSender<TRequest>) -> BoxFuture<()> + Send + Sync>;
pub type ListenerFunctionWrapper = Arc<dyn Fn() -> BoxFuture<()> + Send + Sync>;
pub type HandlerFunction<TRequest> = Arc<dyn Fn(TRequest) -> BoxFuture<()> + Send + Sync>;
pub type MessageHandlerFunction<TContext> = Arc<dyn Fn(TContext) -> BoxFuture<()> + Send + Sync>;
pub type SenderFunction = Arc<dyn Fn(SenderContext) -> BoxFuture<()> + Send + Sync>;