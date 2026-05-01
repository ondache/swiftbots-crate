use swiftbots::{BasicBot, SwiftBots};
use tokio;
use tokio::sync::mpsc::channel;
use swiftbots::types::SwiftBotsError;

#[tokio::test]
async fn base_handler_test() -> Result<(), SwiftBotsError>{
    let (result_tx, mut result_rx) = channel(1);

    let bot = BasicBot::new("test".to_string())
        .listener(async move |tx| {
            let message = "Test Value 1".to_string();
            tx.send(message).unwrap();
        })
        .handler(move |req| {
            let result_tx = result_tx.clone();
            async move {
                let message = req;
                result_tx.send(message + "2").await.unwrap();
            }
        });

    SwiftBots::new()
        .add_bot(bot.build()?)?
        .run()
        .await;

    let should_value = result_rx.recv().await.unwrap();
    assert_eq!(should_value, "Test Value 12");
    Ok(())
}