pub mod app;
pub mod types;

pub use crate::app::SwiftBots;
pub use crate::types::SwiftBotsError;

#[cfg(feature = "event-bot")]
pub mod event_bot;
#[cfg(feature = "event-bot")]
pub use crate::event_bot::bot::EventBot;
#[cfg(feature = "event-bot")]
pub use event_bot::functions::run_once;

#[cfg(feature = "chat")]
pub mod chat;
#[cfg(feature = "chat")]
pub use crate::chat::bot::ChatBot;
#[cfg(feature = "chat")]
pub use crate::chat::context::new_request;

#[cfg(feature = "telegram")]
pub mod telegram;
#[cfg(feature = "telegram")]
pub use crate::telegram::bot::TelegramBot;

#[cfg(feature = "rt-tokio")]
pub mod rt_tokio;