// fn main() {}

use swiftbots::{SwiftBots, ChatBot, new_request};
use tokio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing_subscriber;
use swiftbots::types::SwiftBotsError;

#[tokio::main]
async fn main() -> Result<(), SwiftBotsError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let bot = ChatBot::new("chat bot")
        .listener(|tx| async move {
            loop {
                let msg = read_line().await;
                let request = new_request("".to_string(), "chat user", msg.as_str()).unwrap();
                tx.send(request).unwrap();
            }
        })
        .message_handler(vec!["+", "add"], |req, ctx| async move {
            let message = req.body();
            let args = message
                .split_whitespace()
                .map(|s| s.trim().parse::<i32>())
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap())
                .collect::<Vec<i32>>();
            if args.len() != 2 {
                ctx.reply("Invalid number format. Try `+ 2 2` or `- 70 1`").await;
                return;
            }
            ctx.reply(format!("Result: {}", args.iter().sum::<i32>()).as_str()).await;
        })
        .message_handler(vec!["-", "sub"], |req, ctx| async move {
            let message = req.body();
            let args = message
                .split_whitespace()
                .map(|s| s.trim().parse::<i32>())
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap())
                .collect::<Vec<i32>>();
            if args.len() != 2 {
                ctx.reply("Invalid number format. Try `+ 2 2` or `- 70 1`").await;
                return;
            }
            ctx.reply(format!("Result: {}", args[0] - args[1]).as_str()).await;
        })
        .default_handler(|_req, ctx| async move {
            ctx.reply("[default handler] Unknown command. Try `+ 2 2` or `- 70 1`").await
        })
        .sender(|ctx| async move {
            println!("{}", ctx.message)
        });

    println!("Welcome to the {}!", bot.name);
    println!("Type expression to solve like `+ 2 2` or `- 70 1`");
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
    line
}
