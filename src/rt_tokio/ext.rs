#[cfg(feature = "event-bot")]
use std::sync::Arc;
#[cfg(feature = "event-bot")]
use crate::event_bot::types::{SpawnPolicy, EventBot};

#[cfg(feature = "event-bot")]
impl<M, B> SpawnPolicy<M> for B
where
    M: Send + 'static, // TODO: I could remove Send for wasm
    B: EventBot<Message = M> + Send + Sync + 'static
{
    fn spawn_handle(self: Arc<Self>, message: M)
    {
        #[cfg(target_family = "wasm")]
        tokio::task::spawn_local(async move {
            self.handle(message).await;
        });
        #[cfg(not(target_family = "wasm"))]
        tokio::spawn(async move {
            self.handle(message).await;
        });
    }
}