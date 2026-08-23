use std::sync::Arc;
#[cfg(feature = "basic")]
use crate::basic::bot::{SpawnPolicy, BasicBot};

#[cfg(feature = "basic")]
impl<M, B> SpawnPolicy<M> for B
where
    M: Send + 'static,
    B: BasicBot<Message = M> + 'static
{
    fn spawn_handle(self: Arc<Self>, message: M)
    {
        #[cfg(target_family = "wasm")]
        tokio::task::spawn_local(async move {
            self.clone().handle(message).await;
        });
        #[cfg(not(target_family = "wasm"))]
        tokio::spawn(async move {
            self.clone().handle(message).await;
        });
    }
}