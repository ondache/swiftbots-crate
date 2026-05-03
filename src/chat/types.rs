use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use crate::chat::context::{SendFnContext, ChatContext};
pub use crate::types::BoxFuture;

pub type ListenerFunction<TRequest> = Arc<dyn Fn(UnboundedSender<TRequest>) -> BoxFuture<()> + Send + Sync>;
pub type SenderFunction = Arc<dyn Fn(SendFnContext) -> BoxFuture<()> + Send + Sync>;
pub type MessageHandlerFunction<TRequest> = Arc<dyn Fn(TRequest, ChatContext) -> BoxFuture<()> + Send + Sync>;

pub struct ChatCommand<TRequest> {
    pub commands: Vec<String>,
    pub callback: MessageHandlerFunction<TRequest>
}

