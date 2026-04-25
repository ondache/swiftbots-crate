use std::sync::Arc;
use crate::bot::BotBox;

pub struct Request {
    pub message: String,
}

pub struct MiddlewareContext {
    pub bot_box: Arc<BotBox>,
    pub request: Option<Request>,
}

pub struct Context {
    pub message: String,
}

pub struct HandlerBackContext {}