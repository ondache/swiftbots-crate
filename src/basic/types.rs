use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use crate::types::BoxFuture;

pub type ListenerFunction<TRequest> = Arc<dyn Fn(UnboundedSender<TRequest>) -> BoxFuture<()> + Send + Sync>;
pub type HandlerFunction<TRequest> = Arc<dyn Fn(TRequest) -> BoxFuture<()> + Send + Sync>;