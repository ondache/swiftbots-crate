use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use swiftbots::{SwiftBots};
use swiftbots::chat_bot::types::{ChatBot, MessageHandler};
use swiftbots::event_bot::bot::{EventBot};
use tokio;
use tokio::sync::mpsc::{channel, Sender};
use swiftbots::types::{Bot, SwiftBotsError};

struct Bot1 {
    tx: Sender<String>,
    call_counter: AtomicUsize
}

impl Bot1 {}

impl Bot for Bot1 {
    async fn run(self: Arc<Self>) {
        <Self as EventBot>::run(self)
    }

    fn name(&self) -> &str { "" }

    fn enabled(&self) -> bool {
        self.call_counter.load(Ordering::Relaxed) == 2
    }
}

impl EventBot for Bot1 {
    type Message = String;

    async fn listen(&self) -> Self::Message {
        if self.call_counter.fetch_add(1, Ordering::Relaxed) == 0 {
            "+ 6 9".to_string()
        }
        else {
            "- 10 3".to_string()
        }
    }

    async fn handle(&self, message: Self::Message) {
        todo!()
    }
}

#[async_trait::async_trait]
impl ChatBot for Bot1 {
    fn message_handlers(&self) -> Vec<dyn MessageHandler<Self::Message>> {
        vec![AddHandler{tx: self.tx.clone()}, SubtractHandler{tx: self.tx.clone()}]
    }
}

struct AddHandler {
    tx: Sender<String>,
}

impl MessageHandler<String> for AddHandler {
    fn commands() -> Vec<&'static str> {
        vec!["+", "add"]
    }

    async fn handle(&self, message: String, _command: &str) {
        let args = message
            .split_whitespace()
            .map(|s| s.trim().parse::<i32>())
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<i32>>();
        self.tx.send(format!("{}", args[0] + args[1]).to_string()).await.unwrap();
    }
}

struct SubtractHandler {
    tx: Sender<String>,
}

impl MessageHandler<String> for SubtractHandler {
    fn commands() -> Vec<&'static str> {
        vec!["-", "sub"]
    }

    async fn handle(&self, message: String, _command: &str) {
        let args = message
            .split_whitespace()
            .map(|s| s.trim().parse::<i32>())
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<i32>>();
        self.tx.send(format!("{}", args[0] - args[1]).to_string()).await.unwrap();
    }
}

#[tokio::test]
async fn base_handler_test() -> Result<(), SwiftBotsError>{
    let (result_tx, mut result_rx) = channel(2);
    let bot = Bot1{tx: result_tx, call_counter: AtomicUsize::new(0)};

    SwiftBots::new()
        .add_bot(bot)?
        .run()
        .await;

    assert_eq!(result_rx.len(), 2);
    assert_eq!(result_rx.recv().await.unwrap(), "15");
    assert_eq!(result_rx.recv().await.unwrap(), "7");
    Ok(())
}