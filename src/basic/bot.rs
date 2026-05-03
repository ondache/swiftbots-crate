use std::sync::Arc;
use std::future::Future;

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use tower::{ServiceBuilder, BoxError, ServiceExt, Service};
use tower::util::BoxCloneService;
use tracing::{debug, info};

use crate::types::{BoxFuture, SwiftBotsError};
use crate::basic::types::{ListenerFunction, HandlerFunction};
use crate::middleware::{
    BaseHandler,
    EntryService,
};
use crate::bot::BotBox;

pub struct BasicBot <TRequest> {
    pub name: String,
    pub run_at_startup: bool,
    listener_entry: Option<ListenerFunction<TRequest>>,
    handler_entry: Option<HandlerFunction<TRequest>>,
}

impl <TRequest> BasicBot<TRequest>
where TRequest: Send + Sync + 'static
{
    pub fn new(name: &str) -> Self {
        BasicBot {
            name: name.to_string(),
            run_at_startup: true,
            listener_entry: None,
            handler_entry: None,
        }
    }

    pub fn listener<F, Fut>(mut self, listener_func: F) -> Self
    where
        F: Fn(UnboundedSender<TRequest>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.listener_entry = Some(Arc::new(move |tx| {
            Box::pin(listener_func(tx))
        }));
        self
    }

    pub fn handler<F, Fut>(mut self, handler_func: F) -> Self
    where
        F: Fn(TRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.handler_entry = Some(Arc::new(move |ctx| {
            Box::pin(handler_func(ctx))
        }));
        self
    }

    pub fn build(self) -> Result<Arc<BotBox>, SwiftBotsError> {
        let name = Arc::new(self.name);
        let run_at_startup = self.run_at_startup;
        debug!("Building bot: '{}'", name);
        let listener_entry = self
            .listener_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoListener(name.to_string()))?;
        let handler_entry = self
            .handler_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
        let base_handler = BaseHandler::<TRequest> { bot_entry: handler_entry };
        let service = ServiceBuilder::new()
            // .layer(LoggingLayer)
            .service(EntryService { inner: base_handler })
            .boxed_clone();
        let service_task_factory = Self::get_service_tasks(
            name.clone(),
            service,
            listener_entry,
        );
        Ok(Arc::new(BotBox {
            enabled: run_at_startup,
            name,
            service_task_factory,
            service_handles: Vec::new(),
            onetime_handles: Vec::new(),
        }))
    }

    fn get_service_tasks(
        name: Arc<String>,
        service: BoxCloneService<TRequest, (), BoxError>,
        listener_entry: ListenerFunction<TRequest>,
    ) -> Arc<dyn Fn() -> Vec<BoxFuture<()>> + 'static> {
        info!("get_service_tasks");
        let generator = move || {
            let (tx, rx) = unbounded_channel::<TRequest>();
            let mut tasks: Vec<BoxFuture<()>> = Vec::new();
            tasks.push(Self::get_awaitable_handler(name.clone(), service.clone(), rx));
            tasks.push(listener_entry.clone()(tx));
            tasks
        };
        Arc::new(generator)
    }

    fn get_awaitable_handler(
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