use crate::types::Json;
use std::sync::Arc;
use crate::bot::BotBox;

pub struct MiddlewareContext {
    pub bot_box: Arc<BotBox>,
    pub feed_context: Json,
    pub user_context: Json,
}

pub struct SenderContext {
    pub recipient: String,
    pub message: String,
}

pub struct FeedContext {
    pub data: Json,
}

pub struct Context {
    pub req: Json,
}
