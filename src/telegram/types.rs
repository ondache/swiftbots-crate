use std::time::Duration;
use http::header::{HeaderMap};
pub use serde_json::Value as Json;

#[derive(Clone)]
pub struct FetchOptions {
    pub timeout: Option<Duration>,
    pub headers: Option<HeaderMap>,
    pub ignore_errors: bool,
}

impl Default for FetchOptions {
    fn default() -> Self {
        FetchOptions {
            timeout: None,
            headers: None,
            ignore_errors: false,
        }
    }
}

pub enum TgApiErrorStatus {
    WaitAndRetry,
    BadRequest,
    ShouldShutdown,
}