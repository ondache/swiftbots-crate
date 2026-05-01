use std::collections::HashMap;
use crate::bot::BotBox;
use crate::runner::TaskRunner;
use crate::types::{SwiftBotsError};
use std::sync::Arc;
use tracing::{info, warn};

pub struct SwiftBots {
    bots: HashMap<String, Arc<BotBox>>,
}

impl SwiftBots {
    pub fn new() -> Self {
        SwiftBots {
            bots: HashMap::new()
        }
    }

    pub fn add_bot(mut self, bot: Arc<BotBox>) -> Result<Self, SwiftBotsError> {
        if self.bots.contains_key(bot.name.as_str()) {
            let message = format!("Bot with name {} already exists", bot.name);
            return Err(SwiftBotsError::DuplicateBotName(message));
        }
        info!("Registering bot: {}", bot.name);
        self.bots.insert(bot.name.to_string(), bot);
        Ok(self)
    }

    pub async fn run(self) {
        if self.bots.is_empty() {
            let message = "No bots to run";
            warn!("{}", message);
            return;
        }
        info!("Starting SwiftBots application with {} bots", self.bots.len());
        TaskRunner::new(self.bots)
            .run_app()
            .await;
    }
}

