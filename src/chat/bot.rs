use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tower::{BoxError, ServiceBuilder, ServiceExt, Service};
use tower::util::BoxCloneService;
use http::Request;
use tracing::{debug, info};
use crate::bot::BotBox;
use crate::chat::types::{ListenerFunction, SenderFunction, ChatCommand};
use crate::chat::context::{ChatContext, RoutingMeta, SendFnContext};
use crate::chat::routing::{build_token_trie};
use crate::middleware::{BaseHandler, EntryService};
use crate::chat::middleware::{RoutingLayer, ChatContextLayer};
use crate::types::{BoxFuture, SwiftBotsError};
use crate::chat::handlers::chat_handler_extractor;
use serde_json::Value as JsonValue;

pub struct ChatBot <TBody> {
    pub name: String,
    pub run_at_startup: bool,
    pub error_message: String,
    pub unknown_message: String,
    pub refuse_message: String,
    listener_entry: Option<ListenerFunction<Request<TBody>>>,
    sender_entry: Option<SenderFunction>,
    message_handlers: Vec<ChatCommand<Request<TBody>>>,
}


impl <TBody: BodyTransform> ChatBot <TBody> {
    pub fn new(name: &str) -> Self {
        ChatBot {
            name: name.to_string(),
            run_at_startup: true,
            listener_entry: None,
            sender_entry: None,
            message_handlers: vec![],
            error_message: "Error while processing your request".to_string(),
            unknown_message: "Unknown command".to_string(),
            refuse_message: "You are not allowed to use this command".to_string(),
        }
    }

    pub fn listener<F, Fut>(mut self, listener_func: F) -> Self
    where
        F: Fn(UnboundedSender<Request<TBody>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.listener_entry = Some(Arc::new(move |tx| {
            Box::pin(listener_func(tx))
        }));
        self
    }

    pub fn sender<F, Fut>(mut self, sender_func: F) -> Self
    where
        F: Fn(SendFnContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        self.sender_entry = Some(Arc::new(move |ctx| {
            Box::pin(sender_func(ctx))
        }));
        self
    }

    pub fn message_handler<F, Fut>(mut self, commands: Vec<&str>, handler_func: F) -> Self
    where
        F: Fn(Request<TBody>, ChatContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        let command = ChatCommand {
            commands: commands.into_iter().map(|s| s.to_string()).collect(),
            callback: Arc::new(move |req, ctx| {
                Box::pin(handler_func(req, ctx))
            })
        };
        self.message_handlers.push(command);
        self
    }

    pub fn default_handler<F, Fut>(mut self, handler_func: F) -> Self
    where
        F: Fn(Request<TBody>, ChatContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        let command = ChatCommand {
            commands: vec!["".to_string()],
            callback: Arc::new(move |req, ctx| {
                Box::pin(handler_func(req, ctx))
            })
        };
        self.message_handlers.push(command);
        self
    }

    pub fn build(self) -> Result<Arc<BotBox>, SwiftBotsError> {
        let name = Arc::new(self.name.clone());
        let run_at_startup = self.run_at_startup;
        debug!("Building bot: {}", name);
        let listener_entry = self
            .listener_entry.as_ref()
            .ok_or_else(|| SwiftBotsError::BotHasNoListener(name.to_string()))?
            .clone();
        let sender_entry = self
            .sender_entry.as_ref()
            .ok_or_else(|| SwiftBotsError::BotHasNoSender(name.to_string()))?
            .clone();
        let chat_context_template = self.make_chat_context(sender_entry.clone());
        let token_trie = build_token_trie(self.message_handlers)
            .map_err(|e| SwiftBotsError::InvalidCommand(name.to_string(), e.to_string()))?;
        let base_handler = BaseHandler::<Request<TBody>> {
            bot_entry: chat_handler_extractor(TBody::transform_body)
        };
        let service = ServiceBuilder::new()
            .layer(ChatContextLayer{ctx_template: chat_context_template})
            .layer(RoutingLayer{trie: Arc::new(token_trie)})
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
        service: BoxCloneService<Request<TBody>, (), BoxError>,
        listener_entry: ListenerFunction<Request<TBody>>,
    ) -> Arc<dyn Fn() -> Vec<BoxFuture<()>> + 'static> {
        info!("get_service_tasks");
        let generator = move || {
            let (tx, rx) = unbounded_channel::<Request<TBody>>();
            let mut tasks: Vec<BoxFuture<()>> = Vec::new();
            tasks.push(Self::get_awaitable_handler(name.clone(), service.clone(), rx));
            tasks.push(listener_entry.clone()(tx));
            tasks
        };
        Arc::new(generator)
    }

    fn get_awaitable_handler(
        name: Arc<String>,
        service: BoxCloneService<Request<TBody>, (), BoxError>,
        mut rx: UnboundedReceiver<Request<TBody>>
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

    fn make_chat_context(&self, sender_entry: SenderFunction) -> ChatContext {
        ChatContext::new(
            sender_entry.clone(),
            self.error_message.clone(),
            self.unknown_message.clone(),
            self.refuse_message.clone(),
        )
    }
}

pub trait BodyTransform: Send + Sync + Sized + Clone + 'static {
    fn transform_body(req: Request<Self>) -> Request<Self>;
}

impl BodyTransform for String {
    fn transform_body(req: Request<String>) -> Request<String> {
        let routing_meta = req
            .extensions()
            .get::<RoutingMeta<String>>()
            .expect("RoutingMeta not set in request extensions")
            .clone();
        let (parts, _) = req.into_parts();
        Request::from_parts(parts, routing_meta.arguments)
    }
}

impl BodyTransform for JsonValue {
    fn transform_body(req: Request<JsonValue>) -> Request<JsonValue> {
        let routing_meta = req
            .extensions()
            .get::<RoutingMeta<JsonValue>>()
            .expect("RoutingMeta not set in request extensions")
            .clone();
        let (parts, mut body) = req.into_parts();
        body.as_object_mut().unwrap().insert("arguments".to_string(), JsonValue::String(routing_meta.arguments));
        Request::from_parts(parts, body)
    }
}
