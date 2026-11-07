use std::sync::atomic::{AtomicBool, Ordering};
use swiftbots::{EventBot, SwiftBots};
use tokio;
use tokio::sync::mpsc::{channel, Sender};
use swiftbots::types::SwiftBotsError;

struct Bot1 {
    tx: Sender<String>,
    enabled: AtomicBool
}

#[async_trait::async_trait]
impl EventBot for Bot1 {
    type Message = String;

    fn name(&self) -> &str { "" }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    async fn listen(&self) -> Self::Message {
        self.enabled.fetch_not(Ordering::Relaxed);
        "Test Value 1".to_string()
    }

    async fn handle(&self, message: Self::Message) {
        self.tx.clone().send(message + "2").await.unwrap();
    }
}

#[tokio::test]
async fn base_handler_test() -> Result<(), SwiftBotsError>{
    let (result_tx, mut result_rx) = channel(1);
    let bot = Bot1{tx: result_tx, enabled: AtomicBool::new(true)};

    SwiftBots::new()
        .add_bot(bot)?
        .run()
        .await;

    assert!(!result_rx.is_empty());
    let should_value = result_rx.recv().await.unwrap();
    assert_eq!(should_value, "Test Value 12");
    Ok(())
}