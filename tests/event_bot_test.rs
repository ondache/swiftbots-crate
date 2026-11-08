use std::sync::atomic::{AtomicBool, Ordering};
use swiftbots::{SwiftBots};
use tokio;
use tokio::sync::mpsc::{channel, Sender};
use swiftbots::types::{BotMetadata, SwiftBotsError};
use swiftbots::event_bot::bot::{EventBotRunner};
use swiftbots::event_bot::types::{EventBot, EventHandler, EventListener};

struct Bot1 {
    tx: Sender<String>,
    enabled: AtomicBool
}

impl BotMetadata for Bot1 {
    fn name(&self) -> &str { "" }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl EventHandler<String> for Bot1 {
    async fn handle(&self, message: String) {
        self.tx.clone().send(message + "2").await.unwrap();
    }
}

#[async_trait::async_trait]
impl EventListener<String> for Bot1 {
    async fn listen(&self) -> String {
        self.enabled.fetch_not(Ordering::Relaxed);
        "Test Value 1".to_string()
    }
}

#[async_trait::async_trait]
impl EventBot for Bot1 {
    type Message = String;
}

#[tokio::test]
async fn base_handler_test() -> Result<(), SwiftBotsError>{
    let (result_tx, mut result_rx) = channel(1);
    let bot = Bot1{tx: result_tx, enabled: AtomicBool::new(true)};

    let a = EventBotRunner::new(bot);

    SwiftBots::new()
        .add_bot(a)?
        .run()
        .await;

    assert!(!result_rx.is_empty());
    let fact_value = result_rx.recv().await.unwrap();
    assert_eq!(fact_value, "Test Value 12");
    Ok(())
}