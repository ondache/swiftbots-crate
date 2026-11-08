use std::sync::Arc;
use crate::types::BotMetadata;

pub trait EventBot:
    BotMetadata
    + EventHandler<Self::Message>
    + EventListener<Self::Message>
{
    type Message;
}

#[async_trait::async_trait]
pub trait EventHandler<M>
{
    async fn handle(&self, message: M);
}

#[async_trait::async_trait]
pub trait EventListener<M>
{
    async fn listen(&self) -> M;
}

pub trait SpawnPolicy<M>
{
    fn spawn_handle(self: Arc<Self>, message: M);
}