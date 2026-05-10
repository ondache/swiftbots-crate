use crate::types::{BoxFuture, SwiftBotsError};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tower::{BoxError, Service};
use tower::util::BoxCloneService;

pub struct BotBox {
    pub name: Arc<String>,
    pub enabled: bool,
    pub service_task_factory: Arc<dyn Fn() -> Vec<BoxFuture<()>> + 'static>,
    pub service_handles: Vec<JoinHandle<()>>,
    pub onetime_handles: Vec<JoinHandle<()>>,
}

pub struct OneshotBot<TRequest> {
    pub name: Arc<String>,
    pub service: BoxCloneService<TRequest, (), BoxError>,
}

pub async fn run_once<TRequest>(bot: &mut OneshotBot<TRequest>, request: TRequest) -> Result<(), SwiftBotsError> {
    bot.service.clone().call(request).await.map_err(|e| SwiftBotsError::ServiceCallError(e))
}