use std::time::Duration;
use swiftbots::{BasicBot, BasicRequest, SwiftBots};
use tokio;
use tokio::sync::mpsc::{UnboundedSender, channel};
use serde_json::json;

#[tokio::test]
async fn base_handler_test() {
    let (mut result_tx, mut result_rx) = channel(1);

    let bot = BasicBot::new("test".to_string())
        .listener(async move |tx: UnboundedSender<BasicRequest>| {
            tokio::time::sleep(Duration::from_millis(0)).await;
            let message = "Test Value 1".to_string();
            tx.send(BasicRequest { data: json!({"message": message}) }).unwrap();
        })
        .handler(move |ctx| {
            let result_tx = result_tx.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(0)).await;
                let message = ctx.data["message"].as_str().unwrap().to_string();
                result_tx.send(message + "2").await.unwrap();
            }
        });

    SwiftBots::new()
        .add_bot(bot.build())
        .run()
        .await;

    let should_value = result_rx.recv().await.unwrap();
    assert_eq!(should_value, "Test Value 12");
}