use swiftbots::{Bot, SwiftBots, FeedContext};
use tokio;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde_json::json;

#[tokio::main]
async fn main() {
    let bot = Bot::new("console bot".to_string())
        .listener(async |tx| {
            loop {
                let message = read_line().await;
                tx.send(FeedContext { data: json!({"message": message}) }).await.unwrap();
            }
        })
        .handler(async |ctx| {
            println!("Received message: {}", ctx.req["message"]);
        });

    println!("Welcome to the {}! Type anything and press enter:", bot.name);
    SwiftBots::new()
        .add_bot(bot.build())
        .run()
        .await;
}

async fn read_line() -> String {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("Failed to read line");
    line
}