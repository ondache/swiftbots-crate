use std::sync::Arc;
use crate::event_bot::types::{EventBot, SpawnPolicy};
use crate::types::{BotMetadata, RunnableBot};


pub struct EventBotRunner<B>
where B: EventBot
{
    pub inner: Arc<B>
}

impl <B> EventBotRunner<B>
where B: EventBot
{
    pub fn new(bot: B) -> Self {
        Self { inner: Arc::new(bot) }
    }
}

#[async_trait::async_trait]
impl<B> RunnableBot for EventBotRunner<B>
where
    B: EventBot + SpawnPolicy<B::Message> + Send + Sync,
{
    async fn run(self: Arc<Self>) {
        run_event_bot(self.inner.clone()).await
    }
}

impl<B> BotMetadata for EventBotRunner<B>
where
    B: EventBot + Send + Sync,
{
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn enabled(&self) -> bool {
        self.inner.enabled()
    }
}

pub async fn run_event_bot <B>(bot: Arc<B>)
where
    B: EventBot + SpawnPolicy<<B as EventBot>::Message> + Send + Sync,
{
    while bot.enabled() {
        let message = bot.listen().await;
        bot.clone().spawn_handle(message);
    }
}