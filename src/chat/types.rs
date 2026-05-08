use std::sync::Arc;
use crate::chat::context::{SendFnContext, ChatContext};
pub use crate::basic::types::ListenerFunction;
pub use crate::types::BoxFuture;

pub type SenderFunction = dyn Fn(SendFnContext) -> BoxFuture<()> + Send + Sync;
pub type MessageHandlerFunction<TRequest> = dyn Fn(TRequest, ChatContext) -> BoxFuture<()> + Send + Sync;

pub struct ChatCommand<TRequest> {
    pub commands: Vec<String>,
    pub callback: Arc<MessageHandlerFunction<TRequest>>
}

