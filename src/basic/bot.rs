use std::sync::Arc;
use std::rc::Rc;
use std::future::Future;

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use tower::{ServiceBuilder, BoxError, ServiceExt, Service};
use tower::util::BoxCloneService;
use tracing::{debug, info, trace};

use crate::types::{BoxFuture, SwiftBotsError};
use crate::basic::types::{ListenerFunction, HandlerFunction};
use crate::basic::middleware::{
    BaseHandler,
    EntryService,
};
use crate::bot::{BotBox, OneshotBot};

pub struct BasicBot <TRequest> {
    core: BasicBotCore<TRequest>
}

impl <TRequest> BasicBot<TRequest>
where TRequest: Send + Sync + 'static
{
    pub fn new(name: &str) -> Self {
        BasicBot {
            core: BasicBotCore {
                name: Arc::new(name.to_string()),
                run_at_startup: true,
                listener_entry: None,
                handler_entry: None,
            }
        }
    }

    pub fn run_at_startup(mut self, run_at_startup: bool) -> Self {
        self.core.run_at_startup = run_at_startup;
        self
    }

    pub fn listener<F, Fut>(mut self, listener_func: F) -> Self
    where
        F: Fn(UnboundedSender<TRequest>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.core.set_listener(listener_func);
        self
    }

    pub fn handler<F, Fut>(mut self, handler_func: F) -> Self
    where
        F: Fn(TRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.core.set_handler(handler_func);
        self
    }

    pub fn build(self) -> Result<Rc<BotBox>, SwiftBotsError> {
        let core = self.core;
        let name = core.name;
        debug!("Building bot: '{}'", name);
        let listener_entry = core
            .listener_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoListener(name.to_string()))?;
        let handler_entry = core
            .handler_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
        let base_handler = BaseHandler::<TRequest> { bot_entry: handler_entry };
        let service = ServiceBuilder::new()
            .service(EntryService { inner: base_handler })
            .boxed_clone();
        let service_task_factory = BasicBotCore::get_service_tasks(
            name.clone(),
            service,
            listener_entry,
        );
        Ok(Rc::new(BotBox {
            enabled: core.run_at_startup,
            name,
            service_task_factory,
            service_handles: Vec::new(),
            onetime_handles: Vec::new(),
        }))
    }

    pub fn build_oneshot(self) -> Result<OneshotBot<TRequest>, SwiftBotsError> {
        trace!("BasicBot:build_oneshot");
        let core = &self.core;
        let handler_entry = core
            .handler_entry.clone()
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(core.name.to_string()))?;
        let base_handler = BaseHandler::<TRequest> { bot_entry: handler_entry };
        let service = ServiceBuilder::new()
            .service(EntryService { inner: base_handler })
            .boxed_clone();
        Ok(OneshotBot {
            name: core.name.clone(),
            service,
        })
    }
}

pub struct BasicBotCore<TRequest> {
    pub name: Arc<String>,
    pub run_at_startup: bool,
    pub listener_entry: Option<Arc<ListenerFunction<TRequest>>>,
    pub handler_entry: Option<Arc<HandlerFunction<TRequest>>>,
}

impl <TRequest> BasicBotCore<TRequest>
where TRequest: Send + Sync + 'static {
    pub fn set_listener<F, Fut>(&mut self, listener_func: F)
    where
        F: Fn(UnboundedSender<TRequest>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.listener_entry = Some(Arc::new(move |tx| {
            Box::pin(listener_func(tx))
        }));
    }

    pub fn set_handler<F, Fut>(&mut self, handler_func: F)
    where
        F: Fn(TRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.handler_entry = Some(Arc::new(move |ctx| {
            Box::pin(handler_func(ctx))
        }));
    }

    pub fn get_service_tasks(
        name: Arc<String>,
        service: BoxCloneService<TRequest, (), BoxError>,
        listener_entry: Arc<ListenerFunction<TRequest>>,
    ) -> Arc<dyn Fn() -> Vec<BoxFuture<()>> + 'static> {
        trace!("get_service_tasks");
        let generator = move || {
            let (tx, rx) = unbounded_channel::<TRequest>();
            let tasks: Vec<BoxFuture<()>> = vec![
                Self::get_awaitable_handler(name.clone(), service.clone(), rx),
                listener_entry.clone()(tx)
            ];
            tasks
        };
        Arc::new(generator)
    }

    pub fn get_awaitable_handler(
        name: Arc<String>,
        service: BoxCloneService<TRequest, (), BoxError>,
        mut rx: UnboundedReceiver<TRequest>
    ) -> BoxFuture<()> {
        Box::pin(
            async move {
                loop {
                    if let Some(request) = rx.recv().await {
                        debug!("Bot '{}' received request", name);
                        let mut service = service.clone();
                        tokio::spawn(service.call(request));
                    } else {
                        info!("Bot '{}' is stopped", name);
                        break;
                    }
                }
            }
        )
    }
}