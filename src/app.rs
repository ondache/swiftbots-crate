use std::collections::{HashMap};
use crate::bot::{BotBox, Bot};
use crate::runner::TaskRunner;
use std::sync::Arc;

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
            panic!("Bot with name {} already exists", bot.name);
        }
        self.bots.insert(bot.name.clone(), bot);
        self
    }

    pub async fn run(self) {
        if self.bots.is_empty() {
            panic!("No bots to run");
        }
        let bots = Self::build_bots(self.bots);
        TaskRunner::new(bots)
            .run_app()
            .await;
    }

    fn build_bots(bots: HashMap<String, Bot>) -> HashMap<String, Arc<BotBox>> {
        bots.into_iter()
            .map(|(name, bot)| (name, Arc::new(bot.build())))
            .collect()
    }
}

