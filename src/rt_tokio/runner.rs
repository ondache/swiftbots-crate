use crate::types::Bot;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{info, debug, warn};
#[cfg(target_family = "wasm")]
use tokio::task::LocalSet;

pub struct Supervisor {
    bots: Vec<Arc<dyn Bot>>,
}

impl Supervisor {
    pub fn new(bots: Vec<Arc<dyn Bot>>) -> Self {
        Supervisor {
            bots,
        }
    }

    #[cfg(target_family = "wasm")]
    pub async fn run_app(self) {
        let local = LocalSet::new();

        let mut set = JoinSet::new();
        debug!("App run started");
        info!("Bots for running: {}", self.bots
            .iter()
            .filter(|bot| bot.enabled())
            .map(|bot| bot.name())
            .collect::<Vec<_>>()
            .join(", "));
        for bot in self.bots.iter().filter(|bot| bot.enabled()) {
            set.spawn_local_on(bot.clone().run(), &local);
        }
        local.run_until(set.join_all()).await;
        warn!("All bots are stopped");
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn run_app(self) {
        let mut set = JoinSet::new();
        debug!("App run started");
        info!("Bots for running: {}", self.bots
            .iter()
            .filter(|bot| bot.enabled())
            .map(|bot| bot.name())
            .collect::<Vec<_>>()
            .join(", "));
        for bot in self.bots.iter().filter(|bot| bot.enabled()) {
            set.spawn(bot.clone().run());
        }
        set.join_all().await;
        warn!("All bots are stopped");
    }
}