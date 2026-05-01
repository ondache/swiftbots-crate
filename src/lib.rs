pub mod app;
pub mod bot;
pub mod runner;
pub mod types;
pub mod utils;

pub use crate::app::SwiftBots;

#[cfg(feature = "middleware")]
pub mod middleware;
#[cfg(feature = "basic")]
pub mod basic;
#[cfg(feature = "basic")]
pub use crate::basic::bot::BasicBot;

#[cfg(feature = "chat")]
pub mod chat;