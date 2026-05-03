use std::fmt;
use std::pin::Pin;
use std::future::Future;
use fmt::Display;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

#[derive(Debug)]
pub enum SwiftBotsError {
    DuplicateBotName(String),
    BotHasNoListener(String),
    BotHasNoSender(String),
    BotHasNoHandler(String),
    InvalidCommand(String, String),
}

impl Display for SwiftBotsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwiftBotsError::DuplicateBotName(message) => write!(f, "{message}"),
            SwiftBotsError::BotHasNoListener(name) => write!(f, "Bot '{name}' has no listener"),
            SwiftBotsError::BotHasNoSender(name) => write!(f, "Bot '{name}' has no sender"),
            SwiftBotsError::BotHasNoHandler(name) => write!(f, "Bot '{name}' has no handler"),
            SwiftBotsError::InvalidCommand(name, command) => write!(f, "Invalid command '{command}' for bot '{name}'"),
        }
    }
}