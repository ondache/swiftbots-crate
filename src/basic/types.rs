use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tower::util::BoxCloneService;
use tower::BoxError;
use crate::types::BoxFuture;

pub type ListenerFunction<TRequest> = dyn Fn(UnboundedSender<TRequest>) -> BoxFuture<()> + Send + Sync + 'static;
pub type HandlerFunction<TRequest> = dyn Fn(TRequest) -> BoxFuture<()> + Send + Sync + 'static;

pub struct OneshotBot<TRequest> {
    pub name: Arc<String>,
    pub service: BoxCloneService<TRequest, (), BoxError>,
}