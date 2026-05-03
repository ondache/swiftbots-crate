use std::sync::Arc;
use crate::chat::types::{SenderFunction, MessageHandlerFunction};
use http::{Request, Result};

pub struct SendFnContext {
    pub recipient: String,
    pub message: String,
}

#[derive(Clone)]
pub struct SenderMeta {
    pub sender: String,
    pub message: String,
}

#[derive(Clone)]
pub struct RoutingMeta<TBody> {
    pub arguments: String,
    pub handler: MessageHandlerFunction<Request<TBody>>,
}

#[derive(Clone)]
pub struct ChatContext {
    pub sender: String,
    pub orig_message: String,
    send_fn: SenderFunction,
    error_message: Arc<String>,
    unknown_message: Arc<String>,
    refuse_message: Arc<String>,
}

impl ChatContext {
    pub fn new(send_fn: SenderFunction, error_message: String, unknown_message: String, refuse_message: String) -> Self {
        ChatContext {
            sender: "".to_string(),
            orig_message: "".to_string(),
            send_fn: send_fn,
            error_message: Arc::new(error_message),
            unknown_message: Arc::new(unknown_message),
            refuse_message: Arc::new(refuse_message),
        }
    }

    pub async fn reply(&self, message: &str) {
        let ctx = SendFnContext {
            recipient: self.sender.to_string(),
            message: message.to_string(),
        };
        (self.send_fn)(ctx).await
    }

    pub async fn error(&self) {
        let ctx = SendFnContext {
            recipient: self.sender.to_string(),
            message: self.error_message.to_string(),
        };
        (self.send_fn)(ctx).await
    }

    pub async fn unknown_command(&self) {
        let ctx = SendFnContext {
            recipient: self.sender.to_string(),
            message: self.unknown_message.to_string(),
        };
        (self.send_fn)(ctx).await
    }

    pub async fn refuse_command(&self) {
        let ctx = SendFnContext {
            recipient: self.sender.to_string(),
            message: self.refuse_message.to_string(),
        };
        (self.send_fn)(ctx).await
    }
}

pub fn new_request<TBody>(body: TBody, sender: &str, message: &str) -> Result<Request<TBody>> {
    let feed_ctx = SenderMeta {
        sender: sender.to_string(),
        message: message.to_string(),
    };

    Request::builder()
        .extension(feed_ctx)
        .body(body)
}