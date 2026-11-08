pub mod app;
pub mod types;

pub use crate::app::SwiftBots;
pub use crate::types::SwiftBotsError;

#[cfg(feature = "event-bot")]
pub mod event_bot;

#[cfg(feature = "chat")]
pub mod chat_bot;
#[cfg(feature = "chat")]
pub use crate::chat_bot::bot::ChatBotInstLegacy;
#[cfg(feature = "chat")]
pub use crate::chat_bot::context::new_request;

#[cfg(feature = "telegram")]
pub mod telegram;
#[cfg(feature = "telegram")]
pub use crate::telegram::bot::TelegramBot;

#[cfg(feature = "rt-tokio")]
pub mod rt_tokio;