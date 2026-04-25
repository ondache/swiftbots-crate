use std::time::Duration;
use swiftbots::{Bot, Request, SwiftBots};
use tokio;
use tokio::sync::mpsc;


#[tokio::test]
async fn base_handler_test() {
    let (result_tx, mut result_rx) = mpsc::channel(1);

    let bot = Bot::new("test".to_string())
        .listener(async move |tx: mpsc::Sender<Request>| {
            tokio::time::sleep(Duration::from_millis(0)).await;
            let message = "Test Value 1".to_string();
            tx.send(Request { message }).await.unwrap();
        })
        .handler(move |ctx| {
            let result_tx = result_tx.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(0)).await;
                result_tx.send(ctx.message + "2").await.unwrap();
            }
        });

    SwiftBots::new()
        .add_bot(bot)
        .run()
        .await;

    let should_value = result_rx.recv().await.unwrap();
    assert_eq!(should_value, "Test Value 12");
}