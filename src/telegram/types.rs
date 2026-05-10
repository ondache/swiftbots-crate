use std::time::Duration;
use http::header::{HeaderMap};
pub use serde_json::Value as Json;

#[derive(Clone)]
#[derive(Default)]
pub struct FetchOptions {
    pub timeout: Option<Duration>,
    pub headers: Option<HeaderMap>,
    pub ignore_errors: bool,
}


pub enum TgApiErrorStatus {
    WaitAndRetry,
    BadRequest,
    ShouldShutdown,
}