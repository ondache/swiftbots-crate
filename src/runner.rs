use crate::bot::BotBox;
use crate::context::{MiddlewareContext, Request};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, debug, warn};


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
        debug!("App run started");
        let bots_to_run: Vec<Arc<BotBox>> = self.bots
            .iter()
            .filter(|(_, bot)| bot.enabled)
            .map(|(_, bot)| bot.clone())
            .collect();
        info!("Bots for running: {}", bots_to_run.iter().map(|bot| bot.name.clone()).collect::<Vec<String>>().join(", "));
        let handles: Vec<_> = bots_to_run
            .iter()
            .map(|bot| tokio::spawn(Self::run_bot(bot.clone())))
            .collect();

        for handle in handles {
            let status = handle.await;
            if let Ok(_status) = status {
                warn!("Bot task completed");
            }
            if let Err(err) = status {
                error!("Bot task failed: {:?}", err);
            }
        }

        warn!("All bots are stopped");
    }

    async fn run_bot(bot: Arc<BotBox>) {
        info!("Starting bot: {}", bot.name);
        let (tx, mut rx) = mpsc::channel::<Request>(100);
        tokio::spawn((bot.clone().listener)(tx));

        loop {
            let mut ctx = MiddlewareContext {
                bot_box: bot.clone(),
                user_ctx: None,
                request: None,
            };
            if let Some(request) = rx.recv().await {
                debug!("Bot {} received request", bot.name);
                ctx.request = Some(request);
                tokio::spawn((bot.entry)(ctx));
            } else {
                info!("Bot {} is stopped", bot.name);
                break;
            }
        }
    }
}