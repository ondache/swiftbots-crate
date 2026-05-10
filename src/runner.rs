use crate::bot::BotBox;
use std::collections::HashMap;
use std::rc::Rc;
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
                set.spawn(task);
            }
        }

        set.join_all().await;

        warn!("All bots are stopped");
    }
}