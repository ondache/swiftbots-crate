use crate::bot::BotBox;
use std::collections::HashMap;
use std::rc::Rc;
#[cfg(target_family = "wasm")]
use tokio::task::LocalSet;
use tokio::task::JoinSet;
use tracing::{info, debug, warn};

pub struct TaskRunner {
    bots: HashMap<String, Rc<BotBox>>,
}

impl TaskRunner {
    pub fn new(bots: HashMap<String, Rc<BotBox>>) -> Self {
        TaskRunner {
            bots,
        }
    }

    pub async fn run_app(self) {
        #[cfg(target_family = "wasm")]
        let local = LocalSet::new();
        let mut set = JoinSet::new();
        debug!("App run started");
        let bots_to_run: Vec<Rc<BotBox>> = self.bots
            .iter()
            .filter(|(_, bot)| bot.enabled)
            .map(|(_, bot)| bot.clone())
            .collect();
        info!("Bots for running: {}", bots_to_run
            .iter()
            .map(|bot| bot.name.as_str())
            .collect::<Vec<&str>>()
            .join(", "));
        for bot in bots_to_run.iter() {
            for task in bot.clone().service_task_factory.clone()() {
                #[cfg(target_family = "wasm")]
                set.spawn_local_on(task, &local);
                #[cfg(not(target_family = "wasm"))]
                set.spawn(task);
            }
        }

        #[cfg(target_family = "wasm")]
        local.run_until(set.join_all()).await;
        #[cfg(not(target_family = "wasm"))]
        set.join_all().await;

        warn!("All bots are stopped");
    }
}