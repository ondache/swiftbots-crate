use std::collections::{HashMap};
use crate::bot::BotBox;
use crate::runner::TaskRunner;
use std::sync::Arc;
use tracing::info;

pub struct SwiftBots {
    bots: HashMap<String, Arc<BotBox>>,
}

impl SwiftBots {
    pub fn new() -> Self {
        SwiftBots {
            bots: HashMap::new()
        }
    }

    pub fn add_bot(mut self, bot: Arc<BotBox>) -> Self {
        if self.bots.contains_key(&bot.name) {
            let message = format!("Bot with name {} already exists", bot.name);
            panic!("{}", message);
        }
        info!("Registering bot: {}", bot.name);
        self.bots.insert(bot.name.clone(), bot);
        self
    }

    pub async fn run(self) {
        if self.bots.is_empty() {
            let message = "No bots to run";
            panic!("{}", message);
        }
        info!("Starting SwiftBots application with {} bots", self.bots.len());
        TaskRunner::new(self.bots)
            .run_app()
            .await;
    }
}

