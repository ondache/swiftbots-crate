use std::sync::Arc;
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;
use tower::ServiceBuilder;
use http::Request;
use tracing::{debug, trace};
use crate::chat_bot::context::{ChatContext, RoutingMeta, SendFnContext};
use crate::chat_bot::routing::build_token_trie;
use crate::event_bot::middleware::{box_bot_service, BaseHandler, EntryService};
use crate::chat_bot::middleware::{ChatContextLayer, RoutingLayer};
use crate::types::{SwiftBotsError, MaybeSendFuture, Bot};
use crate::chat_bot::types::{ChatBot, MessageHandler};
use crate::chat_bot::handlers::chat_handler_extractor;
use serde_json::Value as JsonValue;
use crate::event_bot::bot::{EventBot, SpawnPolicy};

pub struct ChatBotWithMiddlewares_TempName {
    pub name: &'static str,
    pub enabled: bool,
}

impl ChatBotWithMiddlewares_TempName {
    pub fn new(name: &'static str) -> Self {
        Self {
            name: name,
            enabled: true,
        }
    }
}

impl<TBody: Send + 'static> EventBot for ChatBotWithMiddlewares_TempName<TBody> {
    type Message = Request<TBody>;

    async fn listen(&self) -> Self::Message {
        
    }

    async fn handle(&self, message: Self::Message) {
        
    }
}

impl<TBody> Bot for ChatBotWithMiddlewares_TempName<TBody> {
    async fn run(self: Arc<Self>) {
        self.run()
    }

    fn name(&self) -> &str {
        self.name
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl <TBody> ChatBot for ChatBotWithMiddlewares_TempName<TBody>
{
    fn message_handlers(&self) -> Vec<dyn MessageHandler<Self::Message>> {
        todo!()
    }
}

pub struct ChatBotWithMiddlewaresBuilder_TempName <TBody> {
    error_message: Option<&'static str>,
    unknown_message: Option<&'static str>,
    refuse_message: Option<&'static str>,
    run_at_startup: bool,
}

impl <TBody> ChatBotWithMiddlewaresBuilder_TempName <TBody> {
    pub fn new(name: &str) -> Self {
        ChatBotWithMiddlewaresBuilder_TempName {
            core: BasicBotCore {
                name: Arc::new(name.to_string()),
                run_at_startup: true,
            },
            chat_core: ChatCore {
                sender_entry: None,
                message_handlers: vec![],
                error_message: "Error while processing your request".to_string(),
                unknown_message: "Unknown command".to_string(),
                refuse_message: "You are not allowed to use this command".to_string(),
            },
            error_message: None,
            unknown_message: None,
            refuse_message: None,
            run_at_startup: true,
        }
    }

    pub fn run_at_startup(mut self, run_at_startup: bool) -> Self {
        self.run_at_startup = run_at_startup;
        self
    }

    pub fn set_error_message(mut self, error_message: &'static str) -> Self {
        self.error_message = Some(error_message);
        self
    }

    pub fn set_unknown_message(mut self, unknown_message: &'static str) -> Self {
        self.unknown_message = Some(unknown_message);
        self
    }

    pub fn set_refuse_message(mut self, refuse_message: &'static str) -> Self {
        self.refuse_message = Some(refuse_message);
        self
    }

    pub fn build(self) -> Result<ChatBotWithMiddlewares_TempName<TBody>, SwiftBotsError> {
        // let core = self.core;
        // let mut chat_core = self.chat_core;
        // let name = core.name.clone();
        chat_core.error_message = self.error_message.unwrap_or(chat_core.error_message);
        chat_core.refuse_message = self.refuse_message.unwrap_or(chat_core.refuse_message);
        chat_core.unknown_message = self.unknown_message.unwrap_or(chat_core.unknown_message);
        debug!("Building bot: {}", name);
        let listener_entry = core
            .listener_entry
            .as_ref()
            .ok_or_else(|| SwiftBotsError::BotHasNoListener(name.to_string()))?
            .clone();
        let sender_entry = chat_core
            .sender_entry.as_ref()
            .ok_or_else(|| SwiftBotsError::BotHasNoSender(name.to_string()))?
            .clone();
        let chat_context_template = chat_core.make_chat_context(sender_entry.clone());
        let token_trie = build_token_trie(chat_core.message_handlers)
            .map_err(|e| SwiftBotsError::InvalidCommand(name.to_string(), e.to_string()))?;
        let handler_entry = core
            .handler_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
        let base_handler = BaseHandler::<Request<TBody>> { bot_entry: handler_entry };
        let service = box_bot_service(ServiceBuilder::new()
            .layer(ChatContextLayer{ctx_template: chat_context_template})
            .layer(RoutingLayer{trie: Arc::new(token_trie)})
            .service(EntryService { inner: base_handler })
        );
        let service_task_factory = BasicBotCore::get_service_tasks(
            name.clone(),
            service,
            listener_entry,
        );
        Ok(Rc::new(BotBox {
            enabled: core.run_at_startup,
            name,
            service_task_factory,
            service_handles: vec![],
            onetime_handles: vec![],
        }))
    }
}

pub struct ChatBotInstLegacy<TBody> {
    core: BasicBotCore<Request<TBody>>,
    chat_core: ChatCore<TBody>,
}


impl <TBody: BodyTransform> ChatBotInstLegacy<TBody> {
    pub fn sender<F, Fut>(mut self, sender_func: F) -> Self
    where
        F: Fn(SendFnContext) -> Fut + Send + Sync + 'static,
        Fut: MaybeSendFuture<Output = ()> + 'static
    {
        self.chat_core.set_sender(sender_func);
        self
    }

    pub fn message_handler<F, Fut>(mut self, commands: Vec<&str>, handler_func: F) -> Self
    where
        F: Fn(Request<TBody>, ChatContext) -> Fut + Send + Sync + 'static,
        Fut: MaybeSendFuture<Output = ()> + 'static
    {
        self.chat_core.append_message_handler(commands, handler_func);
        self
    }

    pub fn default_handler<F, Fut>(mut self, handler_func: F) -> Self
    where
        F: Fn(Request<TBody>, ChatContext) -> Fut + Send + Sync + 'static,
        Fut: MaybeSendFuture<Output = ()> + 'static
    {
        self.chat_core.append_message_handler(vec![""], handler_func);
        self
    }

    pub fn build(self) -> Result<Rc<BotBox>, SwiftBotsError> {
        let core = self.core;
        let mut chat_core = self.chat_core;
        let name = core.name.clone();
        chat_core.error_message = self.error_message.unwrap_or(chat_core.error_message);
        chat_core.refuse_message = self.refuse_message.unwrap_or(chat_core.refuse_message);
        chat_core.unknown_message = self.unknown_message.unwrap_or(chat_core.unknown_message);
        debug!("Building bot: {}", name);
        let listener_entry = core
            .listener_entry
            .as_ref()
            .ok_or_else(|| SwiftBotsError::BotHasNoListener(name.to_string()))?
            .clone();
        let sender_entry = chat_core
            .sender_entry.as_ref()
            .ok_or_else(|| SwiftBotsError::BotHasNoSender(name.to_string()))?
            .clone();
        let chat_context_template = chat_core.make_chat_context(sender_entry.clone());
        let token_trie = build_token_trie(chat_core.message_handlers)
            .map_err(|e| SwiftBotsError::InvalidCommand(name.to_string(), e.to_string()))?;
        let handler_entry = core
            .handler_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
        let base_handler = BaseHandler::<Request<TBody>> { bot_entry: handler_entry };
        let service = box_bot_service(ServiceBuilder::new()
            .layer(ChatContextLayer{ctx_template: chat_context_template})
            .layer(RoutingLayer{trie: Arc::new(token_trie)})
            .service(EntryService { inner: base_handler })
        );
        let service_task_factory = BasicBotCore::get_service_tasks(
            name.clone(),
            service,
            listener_entry,
        );
        Ok(Rc::new(BotBox {
            enabled: core.run_at_startup,
            name,
            service_task_factory,
            service_handles: vec![],
            onetime_handles: vec![],
        }))
    }

    pub fn build_oneshot(self) -> Result<OneshotBot<Request<TBody>>, SwiftBotsError> {
        trace!("ChatBot:build_oneshot");
        let core = self.core;
        let mut chat_core = self.chat_core;
        let name = core.name.clone();
        chat_core.error_message = self.error_message.unwrap_or(chat_core.error_message);
        chat_core.refuse_message = self.refuse_message.unwrap_or(chat_core.refuse_message);
        chat_core.unknown_message = self.unknown_message.unwrap_or(chat_core.unknown_message);
        let sender_entry = chat_core
            .sender_entry.as_ref()
            .ok_or_else(|| SwiftBotsError::BotHasNoSender(name.to_string()))?
            .clone();
        let chat_context_template = chat_core.make_chat_context(sender_entry.clone());
        let token_trie = build_token_trie(chat_core.message_handlers)
            .map_err(|e| SwiftBotsError::InvalidCommand(name.to_string(), e.to_string()))?;
        let handler_entry = core
            .handler_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
        let base_handler = BaseHandler::<Request<TBody>> { bot_entry: handler_entry };
        let service = box_bot_service(ServiceBuilder::new()
            .layer(ChatContextLayer{ctx_template: chat_context_template})
            .layer(RoutingLayer{trie: Arc::new(token_trie)})
            .service(EntryService { inner: base_handler })
        );
        Ok(OneshotBot{
            name: name,
            service: service,
        })
    }
}

pub struct ChatCore <TBody> {
    pub error_message: String,
    pub unknown_message: String,
    pub refuse_message: String,
    pub sender_entry: Option<Arc<SenderFunction>>,
    pub message_handlers: Vec<ChatCommandLegacy<Request<TBody>>>,
}

impl <TBody: BodyTransform> ChatCore <TBody> {
    pub fn set_sender<F, Fut>(&mut self, sender_func: F)
    where
        F: Fn(SendFnContext) -> Fut + Send + Sync + 'static,
        Fut: MaybeSendFuture<Output = ()> + 'static
    {
        self.sender_entry = Some(Arc::new(move |ctx| {
            Box::pin(sender_func(ctx))
        }));
    }

    pub fn append_message_handler<F, Fut>(&mut self, commands: Vec<&str>, handler_func: F)
    where
        F: Fn(Request<TBody>, ChatContext) -> Fut + Send + Sync + 'static,
        Fut: MaybeSendFuture<Output = ()> + 'static
    {
        let command = ChatCommandLegacy {
            commands: commands.into_iter().map(|s| s.to_string()).collect(),
            callback: Arc::new(move |req, ctx| {
                Box::pin(handler_func(req, ctx))
            })
        };
        self.message_handlers.push(command);
    }

    pub fn make_chat_context(&self, sender_entry: Arc<SenderFunction>) -> ChatContext {
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
