use std::time::Duration;
use swiftbots::{Bot, FeedContext, SwiftBots};
use tokio;
use tokio::sync::mpsc::{Sender, channel};
use serde_json::json;

#[tokio::test]
async fn base_handler_test() {
    let (result_tx, mut result_rx) = channel(1);

    let bot = Bot::new("test".to_string())
        .listener(async move |tx: Sender<FeedContext>| {
            tokio::time::sleep(Duration::from_millis(0)).await;
            let message = "Test Value 1".to_string();
            tx.send(FeedContext { data: json!({"message": message}) }).await.unwrap();
        })
        .handler(move |ctx| {
            let result_tx: Sender<String> = result_tx.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(0)).await;
                result_tx.send(ctx.req["message"].as_str().unwrap().to_string() + "2").await.unwrap();
            }
        });

    SwiftBots::new()
        .add_bot(bot.build())
        .run()
        .await;

    let should_value = result_rx.recv().await.unwrap();
    assert_eq!(should_value, "Test Value 12");
}