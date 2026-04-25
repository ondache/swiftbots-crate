use swiftbots::{Bot, Request, SwiftBots};
use tokio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let bot = Bot::new("console bot".to_string())
        .listener(async |tx: mpsc::Sender<Request>| {
            loop {
                let message = read_line().await;
                tx.send(Request { message }).await.unwrap();
            }
        })
        .handler(async |ctx| {
            println!("Received message: {}", ctx.message);
        });

    println!("Welcome to the {}! Type anything and press enter:", bot.name);
    SwiftBots::new()
        .add_bot(bot)
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