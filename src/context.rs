use crate::types::{Json, SenderFunction};
use std::sync::Arc;
use crate::bot::BotBox;

// pub struct MiddlewareContext {
//     pub bot_box: Arc<BotBox>,
//     pub feed_context: Json,
//     pub user_context: Json,
// }

pub struct SenderContext {
    pub recipient: String,
    pub message: String,
}

#[derive(Clone)]
pub struct BasicRequest {
    pub data: Json,
}


// pub struct ChatContext {
//     pub bot_box: Arc<BotBox>,
//     pub req: Json,
//     pub message: String,
//     sender_func: SenderFunction,
// }

// impl ChatContext {
//     pub fn new(bot_box: Arc<BotBox>, req: Json, message: String, sender_func: SenderFunction) -> Self {
//         ChatContext {
//             bot_box,
//             req,
//             message,
//             sender_func,
//         }
//     }
//     pub async fn reply(self, message: String) {
//         
//     }
// }