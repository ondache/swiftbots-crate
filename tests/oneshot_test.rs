use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;
use swiftbots::event_bot::types::{EventBot, EventHandler, EventListener, SpawnPolicy};
use tokio;
use swiftbots::types::{BotMetadata, SwiftBotsError};

struct Bot1 {
    counter: AtomicUsize
}

impl BotMetadata for Bot1 {
    fn name(&self) -> &str { "" }

    fn enabled(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl EventHandler<()> for Bot1 {
    async fn handle(&self, _: ()) {
        self.counter.fetch_add(111, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl EventListener<()> for Bot1 {
    async fn listen(&self) -> () {
        ()
    }
}

#[async_trait::async_trait]
impl EventBot for Bot1 {
    type Message = ();
}

#[tokio::test]
async fn base_handler_test() -> Result<(), SwiftBotsError>{
    let bot = Bot1 {
        counter: AtomicUsize::new(0)
    };
    bot.handle(()).await;

    assert_eq!(bot.counter.load(Ordering::Relaxed), 111);
    Ok(())
}