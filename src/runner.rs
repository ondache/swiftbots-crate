use crate::bot::BotBox;
use crate::context::{MiddlewareContext, Request};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;


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
        let handles: Vec<_> = self.bots
            .values()
            .filter(|bot| bot.enabled)
            .map(|bot| tokio::spawn(Self::run_bot(bot.clone())))
            .collect();

        for handle in handles {
            let status = handle.await;
            if let Ok(_status) = status {
                println!("Bot task completed");
            }
            if let Err(err) = status {
                eprintln!("Bot task failed: {:?}", err);
            }
        }

        println!("All bots are stopped");
    }

    async fn run_bot(bot: Arc<BotBox>) {
        let (tx, mut rx) = mpsc::channel::<Request>(100);
        tokio::spawn((bot.clone().listener)(tx));

        loop {
            let mut ctx = MiddlewareContext {
                bot_box: bot.clone(),
                request: None,
            };
            if let Some(request) = rx.recv().await {
                ctx.request = Some(request);
                tokio::spawn((bot.entry)(ctx));
            } else {
                println!("Bot {} is stopped", bot.bot.name);
                break;
            }
        }
    }
}