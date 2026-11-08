use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[async_trait::async_trait]
pub trait EventBot {
    type Message;
    async fn listen(&self) -> Self::Message;
    async fn handle(&self, message: Self::Message);
    fn name(&self) -> &str;
    fn enabled(&self) -> bool;
}

pub trait SpawnPolicy<M>
{
    fn spawn_handle(self: Arc<Self>, message: M);
}