#[cfg(feature = "rt-tokio")]
use crate::rt_tokio::Supervisor;
use crate::types::{SwiftBotsError, Bot};
use std::sync::Arc;
use tracing::{info};
#[cfg(feature = "rt-tokio")]
use tracing::{error};

pub struct SwiftBots {
    bots: Vec<Arc<dyn Bot>>,
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

    pub fn add_bot<T: Bot + 'static>(mut self, bot: T) -> Result<Self, SwiftBotsError> {
        if self.bots.iter().any(|bot| bot.name() == bot.name()) {
            let message = format!("Bot with name {} already exists", bot.name());
            return Err(SwiftBotsError::DuplicateBotName(message));
        }
        info!("Registering bot: {}", bot.name());
        self.bots.push(Arc::new(bot));
        Ok(self)
    }

    #[cfg(feature = "rt-tokio")]
    pub async fn run(self) {
        if self.bots.is_empty() {
            error!("No bots to run");
            return;
        }
        info!("Starting SwiftBots application with {} bots", self.bots.len());
        Supervisor::new(self.bots)
            .run_app()
            .await;
    }
}

