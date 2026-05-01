use crate::bot::{BotBox};
use crate::context::{BasicRequest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::task::JoinSet;
use tracing::{info, error, debug, warn};
use serde_json::json;
use tower::{Layer, BoxError, Service, ServiceExt, ServiceBuilder};

pub struct TaskRunner {
    bots: HashMap<String, Arc<BotBox>>,
}

impl TaskRunner {
    pub fn new(bots: HashMap<String, Arc<BotBox>>) -> Self {
        TaskRunner {
            bots,
        }
    }

    pub async fn run_app(self) {
        let mut set = JoinSet::new();
        debug!("App run started");
        let bots_to_run: Vec<Arc<BotBox>> = self.bots
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
            // let mut handles = &bot.service_handles;
            for task in bot.clone().service_task_factory.clone()() {
                // let handle = tokio::spawn(task);
                // handles.push(handle.clone());
                set.spawn(task);
            }
        }

        let output = set.join_all().await;

        warn!("All bots are stopped");
    }
}