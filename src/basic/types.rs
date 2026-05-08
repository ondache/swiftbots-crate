use tokio::sync::mpsc::UnboundedSender;
use crate::types::BoxFuture;

pub type ListenerFunction<TRequest> = dyn Fn(UnboundedSender<TRequest>) -> BoxFuture<()> + Send + Sync + 'static;
pub type HandlerFunction<TRequest> = dyn Fn(TRequest) -> BoxFuture<()> + Send + Sync + 'static;