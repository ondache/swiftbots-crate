use tower::Service;
use crate::basic::types::OneshotBot;
use crate::types::SwiftBotsError;

pub async fn run_once<TRequest>(bot: &mut OneshotBot<TRequest>, request: TRequest) -> Result<(), SwiftBotsError> {
    bot.service.clone().call(request).await.map_err(SwiftBotsError::ServiceCallError)
}