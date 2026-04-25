use std::collections::{HashMap};
use crate::bot::{BotBox, Bot};
use crate::runner::TaskRunner;
use std::sync::Arc;
use tracing::{debug, info};

pub struct SwiftBots {
    bots: HashMap<String, Bot>,
}

impl SwiftBots {
    pub fn new() -> Self {
        SwiftBots {
            bots: HashMap::new()
        }
    }

    pub fn add_bot(mut self, bot: Bot) -> Self {
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
        let bots = Self::build_bots(self.bots);
        TaskRunner::new(bots)
            .run_app()
            .await;
    }

    fn build_bots(bots: HashMap<String, Bot>) -> HashMap<String, Arc<BotBox>> {
        debug!("Building bots");
        bots.into_iter()
            .map(|(name, bot)| (name, Arc::new(bot.build())))
            .collect()
    }
}

