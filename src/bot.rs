use std::sync::Arc;
use crate::context::{BasicRequest, SenderContext};
use std::future::Future;
use std::pin::Pin;
use serde_json::json;
use crate::types::BoxFuture;
use crate::types::{ListenerFunction, HandlerFunction, MessageHandlerFunction, SenderFunction, ListenerFunctionWrapper};
use crate::middleware::{
    BaseHandler,
    EntryService,
};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tower::{ServiceBuilder, BoxError, ServiceExt, Service};
use tower::util::BoxCloneService;
use tracing::{debug, info};

pub struct BasicBot {
    pub name: String,
    pub run_at_startup: bool,
    listener_entry: Option<ListenerFunction<BasicRequest>>,
    handler_entry: Option<HandlerFunction<BasicRequest>>,
    // middlewares: Option<Vec<Middleware>>,
}

// pub struct ChatBot {
//     pub name: String,
//     pub run_at_startup: bool,
//     listener_entry: Option<ListenerFunction>,
//     sender_entry: Option<SenderFunction>,
//     middlewares: Option<Vec<Middleware>>,
//     message_handlers: Vec<MessageHandlerFunction>,
// }

impl BasicBot {
    pub fn new(name: String) -> Self {
        BasicBot {
            name,
            run_at_startup: true,
            listener_entry: None,
            handler_entry: None,
        }
    }

    pub fn listener<F, Fut>(mut self, listener_func: F) -> Self
        where
            F: Fn(UnboundedSender<BasicRequest>) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static
    {
        self.listener_entry = Some(Arc::new(move |tx| {
            Box::pin(listener_func(tx))
        }));
        self
    }

    pub fn handler<F, Fut>(mut self, handler_func: F) -> Self
        where
            F: Fn(BasicRequest) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static
    {
        self.handler_entry = Some(Arc::new(move |ctx| {
            Box::pin(handler_func(ctx))
        }));
        self
    }

    pub fn build(mut self) -> Arc<BotBox> {
        let name = Arc::new(self.name);
        let run_at_startup = self.run_at_startup;
        debug!("Building bot: {}", name);
        let listener_entry = Arc::new(self.listener_entry.unwrap_or_else(|| {
            let msg = format!("Bot {} has no listener", name);
            panic!("{}", msg);
        }));
        let handler_entry = self.handler_entry.unwrap_or_else(|| {
            let msg = format!("Bot {} has no handler", name);
            panic!("{}", msg);
        });
        let base_handler = BaseHandler::<BasicRequest> { bot_entry: handler_entry };
        let _service = ServiceBuilder::new()
            // .layer(LoggingLayer)
            .service(EntryService { inner: base_handler })
            .boxed_clone();
        let service_task_factory = Self::get_service_tasks(
            name.clone(),
            _service,
            listener_entry,
        );
        Arc::new(BotBox {
            enabled: run_at_startup,
            name,
            service_task_factory,
            service_handles: Vec::new(),
        })
    }

    fn get_service_tasks(
        name: Arc<String>,
        service: BoxCloneService<BasicRequest, (), BoxError>,
        listener_entry: Arc<ListenerFunction<BasicRequest>>,
    ) -> Arc<dyn Fn() -> Vec<BoxFuture<()>> + 'static> {
        info!("get_service_tasks");
        let generator = move || {
            let (tx, rx) = unbounded_channel::<BasicRequest>();
            let mut tasks: Vec<BoxFuture<()>> = Vec::new();
            tasks.push(Self::get_awaitable_handler(name.clone(), service.clone(), rx));
            tasks.push(listener_entry.clone()(tx));
            tasks
        };
        Arc::new(generator)
    }

    fn get_awaitable_handler(
        name: Arc<String>,
        service: BoxCloneService<BasicRequest, (), BoxError>,
        mut rx: UnboundedReceiver<BasicRequest>
    ) -> BoxFuture<()> {
        Box::pin(
            async move {
                loop {
                    let mut service = service.clone();
                    if let Some(request) = rx.recv().await {
                        debug!("Bot {} received request", name);
                        tokio::spawn(service.call(request));
                    } else {
                        info!("Bot {} is stopped", name);
                        break;
                    }
                }
            }
        )
    }
}

// impl ChatBot {
//     pub fn new(name: String) -> Self {
//         ChatBot {
//             name,
//             run_at_startup: true,
//             message_handlers: vec![],
//             sender_entry: None,
//             listener_entry: None,
//             middlewares: None,
//         }
//     }
// 
//     fn configure_middlewares(&mut self) {
//         debug!("Configuring middlewares");
// 
//         self.middlewares = Some(vec![
//             from_fn(trace_middleware),
//             from_fn(copy_user_context_middleware),
//             // self.get_make_chat_middleware(),
//         ])
//     }
// 
//     fn get_make_chat_middleware(&self) -> Middleware {
//         let sender_func = self.sender_entry.unwrap_or_else(|| {
//             let msg = format!("Bot {} has no sender function set", self.name);
//             panic!("{}", msg);
//         });
//         Arc::new(async move |ctx, next| {
//             debug!("make_chat_middleware");
//             let chat_context = ChatContext::new(
//                 ctx.bot_box.clone(),
//                 ctx.user_context.clone(),
//                 ctx.feed_context.get("message").unwrap().as_str().unwrap().to_string(),
//                 sender_func.clone(),
//             );
//             next(ctx).await
//         })
//     }
// 
//     pub fn listener<F, Fut>(mut self, listener_func: F) -> Self
//     where
//         F: Fn(mpsc::Sender<FeedContext>) -> Fut + Send + Sync + 'static,
//         Fut: Future<Output = ()> + Send + 'static
//     {
//         self.listener_entry = Some(Arc::new(move |tx| {
//             Box::pin(listener_func(tx))
//         }));
//         self
//     }
// 
//     pub fn sender<F, Fut>(mut self, sender_func: F) -> Self
//     where
//         F: Fn(SenderContext) -> Fut + Send + Sync + 'static,
//         Fut: Future<Output = ()> + Send + 'static
//     {
//         self.sender_entry = Some(Arc::new(move |ctx| {
//             Box::pin(sender_func(ctx))
//         }));
//         self
//     }
// 
//     pub fn message_handler<F, Fut>(mut self, commands: Vec<&str>, handler_func: F) -> Self
//     where
//         F: Fn(ChatContext) -> Fut + Send + Sync + 'static,
//         Fut: Future<Output = ()> + Send + 'static
//     {
//         self.message_handlers.push(Arc::new(move |ctx| {
//             Box::pin(handler_func(ctx))
//         }));
//         self
//     }
// 
//     pub fn build(mut self) -> Arc<BotBox> {
//         debug!("Building bot: {}", self.name);
//         self.configure_middlewares();
//         let entry = compose_middlewares(self.middlewares.unwrap_or_else(|| {
//             let msg = format!("Bot {} has no middlewares set", self.name);
//             panic!("{}", msg);
//         }));
//         Arc::new(BotBox {
//             listener: self.listener_entry.unwrap_or_else(|| {
//                 let msg = format!("Bot {} has no listener", self.name);
//                 panic!("{}", msg);
//             }),
//             handler: None,
//             enabled: self.run_at_startup,
//             entry,
//             name: self.name,
//         })
//     }
// }

pub struct BotBox {
    pub name: Arc<String>,
    pub enabled: bool,
    pub service_task_factory: Arc<dyn Fn() -> Vec<BoxFuture<()>> + 'static>,
    pub service_handles: Vec<JoinHandle<()>>,
}
