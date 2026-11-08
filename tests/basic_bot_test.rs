use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use swiftbots::types::{Bot, SwiftBotsError};
use swiftbots::SwiftBots;
use tokio;
use tokio::sync::mpsc::{Sender, channel};

struct Bot1 {
    tx: Sender<usize>,
    enabled: bool,
}

#[async_trait::async_trait]
impl Bot for Bot1 {
    fn name(&self) -> &str { "" }

    fn enabled(&self) -> bool {
        self.enabled
    }

    async fn run(self: Arc<Self>) {
        self.tx.clone().send(11).await.unwrap();
    }
}

#[tokio::test]
async fn base_handler_test() -> Result<(), SwiftBotsError>{
    let (result_tx, mut result_rx) = channel(1);
    let bot = Bot1 { tx: result_tx, enabled: true };

    SwiftBots::new()
        .add_bot(bot)?
        .run()
        .await;

    assert!(!result_rx.is_empty());
    let fact_value = result_rx.recv().await.unwrap();
    assert_eq!(fact_value, 11);
    Ok(())
}