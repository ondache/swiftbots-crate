use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use tokio::sync::mpsc;
use serde_json::Value;
use crate::context::{MiddlewareContext, SenderContext, FeedContext, Context};

pub type Json = Value;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type ListenerFunction = Arc<dyn Fn(mpsc::Sender<FeedContext>) -> BoxFuture<()> + Send + Sync>;
pub type HandlerFunction = Arc<dyn Fn(Context) -> BoxFuture<()> + Send + Sync>;
pub type SenderFunction = Arc<dyn Fn(SenderContext) -> BoxFuture<()> + Send + Sync>;
pub type CallNextMiddleware = Arc<dyn Fn(MiddlewareContext) -> BoxFuture<()> + Send + Sync + 'static>;
pub type Middleware = Arc<dyn Fn(MiddlewareContext, CallNextMiddleware) -> BoxFuture<()> + Send + Sync + 'static>;