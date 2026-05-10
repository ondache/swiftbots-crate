use swiftbots::{SwiftBots, TelegramBot};
use tokio;
use std::env;
use tracing_subscriber;
use swiftbots::types::SwiftBotsError;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), SwiftBotsError> {
    let filter = EnvFilter::new("info,swiftbots=trace");
    fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(filter)
        .init();
    let token = env::var("TOKEN").expect("TOKEN environment variable is not set");
    let bot = TelegramBot::new("tg bot", token.as_str())
        .message_handler(vec!["+", "add"], |req, ctx| async move {
            let message = req.body()["arguments"].as_str().unwrap();
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
            let message = req.body()["arguments"].as_str().unwrap();
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
        });

    SwiftBots::new()
        .add_bot(bot.build()?)?
        .run()
        .await;
    Ok(())
}