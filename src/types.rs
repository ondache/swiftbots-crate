use std::fmt;
use std::pin::Pin;
use std::future::Future;
use fmt::Display;

#[cfg(not(target_family = "wasm"))]
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
#[cfg(target_family = "wasm")]
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

#[cfg(not(target_family = "wasm"))]
pub type BoxResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;
#[cfg(target_family = "wasm")]
pub type BoxResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>>>>;

#[cfg(not(target_family = "wasm"))]
pub trait MaybeSendFuture: Future + Send {}
#[cfg(not(target_family = "wasm"))]
impl<T> MaybeSendFuture for T where T: Future + Send {}

#[cfg(target_family = "wasm")]
pub trait MaybeSendFuture: Future {}
#[cfg(target_family = "wasm")]
impl<T> MaybeSendFuture for T where T: Future {}


#[derive(Debug)]
pub enum SwiftBotsError {
    DuplicateBotName(String),
    BotHasNoListener(String),
    BotHasNoSender(String),
    BotHasNoHandler(String),
    InvalidCommand(String, String),
    HttpError(String),
    ServiceCallError(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for SwiftBotsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwiftBotsError::DuplicateBotName(message) => write!(f, "{message}"),
            SwiftBotsError::BotHasNoListener(name) => write!(f, "Bot '{name}' has no listener"),
            SwiftBotsError::BotHasNoSender(name) => write!(f, "Bot '{name}' has no sender"),
            SwiftBotsError::BotHasNoHandler(name) => write!(f, "Bot '{name}' has no handler"),
            SwiftBotsError::InvalidCommand(name, command) => write!(f, "Invalid command '{command}' for bot '{name}'"),
            SwiftBotsError::HttpError(error) => write!(f, "HTTP error: {error}"),
            SwiftBotsError::ServiceCallError(error) => write!(f, "Service call error: {}", error),
        }
    }
}