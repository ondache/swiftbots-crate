use std::sync::Arc;
use crate::event_bot::types::{EventBot, SpawnPolicy};
// use std::rc::Rc;

// use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
// use tower::{Service, ServiceBuilder};
// use tracing::{debug, info, trace};

// use crate::types::{BoxFuture, MaybeSendFuture, SwiftBotsError};
// use crate::event_bot::types::{HandlerFunction, ListenerFunction, OneshotBot};
// use crate::event_bot::middleware::{box_bot_service, BaseHandler, BotService, EntryService};
use crate::types::Bot;


pub struct EventBotRunner<B>
where B: EventBot + SpawnPolicy<<B as EventBot>::Message>
{
    pub inner: Arc<B>
}

impl <B> EventBotRunner<B>
where B: EventBot + SpawnPolicy<<B as EventBot>::Message>
{
    pub fn new(bot: B) -> Self {
        Self { inner: Arc::new(bot) }
    }
}

#[async_trait::async_trait]
impl<B> Bot for EventBotRunner<B>
where
    B: EventBot + SpawnPolicy<<B as EventBot>::Message> + Send + Sync,
{
    async fn run(self: Arc<Self>) {
        run_event_bot(self.inner.clone()).await
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn enabled(&self) -> bool {
        self.inner.enabled()
    }
}

pub async fn run_event_bot <B>(bot: Arc<B>)
where
    B: EventBot + SpawnPolicy<<B as EventBot>::Message> + Send + Sync,
{
    loop {
        if !bot.enabled() {
            break;
        }
        let message = bot.listen().await;
        bot.clone().spawn_handle(message);
    }
}

// impl <T> EventBot for T
// where T: Bot + SpawnPolicy<Self::Message> {
//     type Message = Self::Message;
// 
//     async fn listen(&self) -> Self::Message {
//         self.listen()
//     }
// 
//     async fn handle(&self, message: Self::Message) {
//         self.handle(message)
//     }
// }

// impl <TRequest> BotCoreBuildable for BasicBot<TRequest>
// where TRequest: Send + Sync + 'static {
//     fn build(self) -> Result<Arc<Self>, SwiftBotsError> {
//         let core = self.core;
//         let name = core.name;
//         debug!("Building bot: '{}'", name);
//         let Some(listener_entry) = core.listener_entry else {
//             return Err(SwiftBotsError::BotHasNoListener(name.to_string()));
//         };
//         let handler_entry = core
//             .handler_entry
//             .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
//         let base_handler = BaseHandler::<TRequest> { bot_entry: handler_entry };
//         let service = box_bot_service(ServiceBuilder::new()
//             .service(EntryService { inner: base_handler })
//         );
//         let service_task_factory = BasicBotCore::get_service_tasks(
//             name.clone(),
//             service,
//             listener_entry,
//         );
//         Ok(Rc::new(BotBox {
//             enabled: core.run_at_startup,
//             name,
//             service_task_factory,
//             service_handles: Vec::new(),
//             onetime_handles: Vec::new(),
//         }))
//     }
// }
// 
// impl <TRequest> BasicBot<TRequest>
// where TRequest: Send + Sync + 'static
// {
//     pub fn new(name: &str) -> Self {
//         BasicBot {
//             core: BasicBotCore {
//                 name: Arc::new(name.to_string()),
//                 run_at_startup: true,
//                 listener_entry: None,
//                 handler_entry: None,
//             }
//         }
//     }
// 
//     pub fn run_at_startup(mut self, run_at_startup: bool) -> Self {
//         self.core.run_at_startup = run_at_startup;
//         self
//     }
// 
//     pub fn listener<F, Fut>(mut self, listener_func: F) -> Self
//     where
//         F: Fn(UnboundedSender<TRequest>) -> Fut + Send + Sync + 'static,
//         Fut: MaybeSendFuture<Output = ()> + 'static
//     {
//         self.core.set_listener(listener_func);
//         self
//     }
// 
//     pub fn handler<F, Fut>(mut self, handler_func: F) -> Self
//     where
//         F: Fn(TRequest) -> Fut + Send + Sync + 'static,
//         Fut: MaybeSendFuture<Output = ()> + 'static
//     {
//         self.core.set_handler(handler_func);
//         self
//     }
// 
//     pub fn build_oneshot(self) -> Result<OneshotBot<TRequest>, SwiftBotsError> {
//         trace!("BasicBot:build_oneshot");
//         let core = &self.core;
//         let handler_entry = core
//             .handler_entry.clone()
//             .ok_or_else(|| SwiftBotsError::BotHasNoHandler(core.name.to_string()))?;
//         let base_handler = BaseHandler::<TRequest> { bot_entry: handler_entry };
//         let service = box_bot_service(ServiceBuilder::new()
//             .service(EntryService { inner: base_handler })
//         );
//         Ok(OneshotBot {
//             name: core.name.clone(),
//             service,
//         })
//     }
// }