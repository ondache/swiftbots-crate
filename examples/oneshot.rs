use swiftbots::BasicBot;
use tokio;
use swiftbots::basic::functions::run_once;
use swiftbots::types::SwiftBotsError;


#[tokio::main]
async fn main() -> Result<(), SwiftBotsError>{
    let bot: BasicBot<String> = BasicBot::new("console bot")
        .handler(async |req| {
            println!("Received message: {}", req);
        });
    let mut built = bot.build_oneshot()?;

    let request = "Hello From Oneshot!".to_string();
    run_once(&mut built, request).await?;
    Ok(())
}
