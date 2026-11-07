use swiftbots::EventBot;
use tokio;
use tokio::io::{AsyncBufReadExt, BufReader};
use swiftbots::event_bot::functions::run_once;
use swiftbots::types::SwiftBotsError;

struct EchoBot {
    name: &'static str,
    enabled: bool
}

impl EchoBot {
    fn new(name: &'static str) -> Self {
        EchoBot { name, enabled: true }
    }
}

#[async_trait::async_trait]
impl EventBot for EchoBot {
    type Message = String;

    fn name(&self) -> &str {
        self.name
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    async fn listen(&self) -> Self::Message {
        let message = read_line().await;
        message
    }

    async fn handle(&self, message: Self::Message) {
        println!("Received message: {}", message);
    }
}

#[tokio::main]
async fn main() -> Result<(), SwiftBotsError>{
    let bot = EchoBot::new("echo bot");

    let request = "Hello From Oneshot!".to_string();
    bot.handle(request).await;
    Ok(())
}

async fn read_line() -> String {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("Failed to read line");
    line.trim().to_string()
}