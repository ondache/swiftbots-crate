use crate::types::BoxFuture;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct BotBox {
    pub name: Arc<String>,
    pub enabled: bool,
    pub service_task_factory: Arc<dyn Fn() -> Vec<BoxFuture<()>> + 'static>,
    pub service_handles: Vec<JoinHandle<()>>,
    pub onetime_handles: Vec<JoinHandle<()>>,
}