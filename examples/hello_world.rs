use swiftbots::{BasicBot, SwiftBots};
use tokio;
use tokio::io::{AsyncBufReadExt, BufReader};
use swiftbots::types::SwiftBotsError;

#[tokio::main]
async fn main() -> Result<(), SwiftBotsError>{
    let bot = BasicBot::new("console bot")
        .listener(async |tx| {
            loop {
                let message = read_line().await;
                tx.send(message).unwrap();
            }
        })
        .handler(async |req| {
            println!("Received message: {}", req);
        });

    println!("Welcome to the {}! Type anything and press enter:", bot.name);
    SwiftBots::new()
        .add_bot(bot.build()?)?
        .run()
        .await;
    Ok(())
}

async fn read_line() -> String {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("Failed to read line");
    line.trim().to_string()
}