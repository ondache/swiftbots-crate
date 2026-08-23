use crate::types::Bot;
use std::sync::Arc;
#[cfg(target_family = "wasm")]
use tokio::task::LocalSet;
#[cfg(not(target_family = "wasm"))]
use tokio::task::JoinSet;
use tracing::{info, debug, warn};

pub struct Supervisor {
    bots: Vec<Arc<dyn Bot>>,
}

impl Supervisor {
    pub fn new(bots: Vec<Arc<dyn Bot>>) -> Self {
        Supervisor {
            bots,
        }
    }

    pub async fn run_app(self) {
        #[cfg(target_family = "wasm")]
        let local = LocalSet::new();
        #[cfg(not(target_family = "wasm"))]
        let mut set = JoinSet::new();
        debug!("App run started");
        info!("Bots for running: {}", self.bots
            .iter()
            .filter(|bot| bot.enabled())
            .map(|bot| bot.name())
            .collect::<Vec<_>>()
            .join(", "));
        for bot in self.bots.iter().filter(|bot| bot.enabled()) {
            #[cfg(target_family = "wasm")]
            set.spawn_local_on(bot.clone().start(), &local);
            #[cfg(not(target_family = "wasm"))]
            set.spawn(bot.clone().run());
        }

        #[cfg(target_family = "wasm")]
        local.run_until(set.join_all()).await;
        #[cfg(not(target_family = "wasm"))]
        set.join_all().await;

        warn!("All bots are stopped");
    }
}