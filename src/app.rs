#[cfg(feature = "rt-tokio")]
use crate::rt_tokio::TaskRunner;
use crate::types::{SwiftBotsError, BotCore};
use std::sync::Arc;
use tracing::{info, error};

pub struct SwiftBots {
    bots: Vec<Arc<dyn BotCore>>,
}

impl Default for SwiftBots {
    fn default() -> Self {
        Self::new()
    }
}

impl SwiftBots {
    pub fn new() -> Self {
        SwiftBots {
            bots: Vec::new()
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        SwiftBots {
            bots: Vec::with_capacity(size)
        }
    }

    pub fn add_bot(mut self, bot: Arc<dyn BotCore>) -> Result<Self, SwiftBotsError> {
        if self.bots.iter().any(|bot| bot.name() == bot.name()) {
            let message = format!("Bot with name {} already exists", bot.name());
            return Err(SwiftBotsError::DuplicateBotName(message));
        }
        info!("Registering bot: {}", bot.name());
        self.bots.push(bot);
        Ok(self)
    }

    #[cfg(feature = "rt-tokio")]
    pub async fn run(self) {
        if self.bots.is_empty() {
            error!("No bots to run");
            return;
        }
        info!("Starting SwiftBots application with {} bots", self.bots.len());
        TaskRunner::new(self.bots)
            .run_app()
            .await;
    }
}

