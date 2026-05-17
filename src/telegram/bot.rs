use std::sync::{Arc, LazyLock};
use std::rc::Rc;
use std::time::Duration;
use tower::ServiceBuilder;
use http::Request;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, error, info, info_span, trace, warn, Instrument};
use serde_json::Value as Json;
use serde_json::json;
use reqwest;
use tokio::time::sleep;
use crate::bot::BotBox;
use crate::basic::bot::BasicBotCore;
use crate::chat::context::{ChatContext, SendFnContext};
use crate::chat::routing::build_token_trie;
use crate::chat::bot::{BodyTransform, ChatCore};
use crate::chat::middleware::{ChatContextLayer, RoutingLayer};
use crate::basic::middleware::{box_bot_service, BaseHandler, EntryService};
use crate::basic::types::OneshotBot;
use crate::types::{MaybeSendFuture, SwiftBotsError};
use crate::chat::handlers::chat_handler_extractor;
use crate::telegram::context::UpdateMeta;
use crate::telegram::helpers::{standard_listener, standard_sender};
use crate::telegram::middleware::DeconstructTgMessageLayer;
use crate::telegram::types::{FetchOptions, TgApiErrorStatus};

static TEXT_WRAPPER: LazyLock<textwrap::Options> = LazyLock::new(|| 
    textwrap::Options::new(4096usize)
        .word_separator(textwrap::WordSeparator::UnicodeBreakProperties)
        .line_ending(textwrap::LineEnding::CRLF)
);

pub struct TelegramBot {
    core: BasicBotCore<Request<Json>>,
    chat_core: ChatCore<Json>,
    tg_core: TelegramCore,

    pub error_message: Option<String>,
    pub unknown_message: Option<String>,
    pub refuse_message: Option<String>,
    pub allowed_updates: Vec<String>,
    pub skip_updates: bool,
}


impl TelegramBot {
    pub fn new(name: &str, token: &str) -> Self {
        let tg_core = TelegramCore {
            token: token.to_string(),
            http_client: reqwest::ClientBuilder::new()
                .build()
                .unwrap_or_else(|e| panic!("Failed to create HTTP client: {}", e)),
            allowed_updates: vec![],
            skip_updates: false,
        };
        let core = BasicBotCore {
            name: Arc::new(name.to_string()),
            run_at_startup: true,
            listener_entry: None,
            handler_entry: Some(chat_handler_extractor(Json::transform_body)),
        };
        let chat_core = ChatCore {
            sender_entry: None,
            message_handlers: vec![],
            error_message: "Error while processing your request".to_string(),
            unknown_message: "Unknown command".to_string(),
            refuse_message: "You are not allowed to use this command".to_string(),
        };
        TelegramBot {
            core: core,
            chat_core: chat_core,
            tg_core: tg_core,
            error_message: None,
            unknown_message: None,
            refuse_message: None,
            skip_updates: false,
            allowed_updates: vec!["messages".to_string()],
        }
    }

    pub fn run_at_startup(mut self, run_at_startup: bool) -> Self {
        self.core.run_at_startup = run_at_startup;
        self
    }

    pub fn message_handler<F, Fut>(mut self, commands: Vec<&str>, handler_func: F) -> Self
    where
        F: Fn(Request<Json>, ChatContext) -> Fut + Send + Sync + 'static,
        Fut: MaybeSendFuture<Output = ()> + 'static
    {
        self.chat_core.append_message_handler(commands, handler_func);
        self
    }

    pub fn default_handler<F, Fut>(mut self, handler_func: F) -> Self
    where
        F: Fn(Request<Json>, ChatContext) -> Fut + Send + Sync + 'static,
        Fut: MaybeSendFuture<Output = ()> + 'static
    {
        self.chat_core.append_message_handler(vec![""], handler_func);
        self
    }

    pub fn build(self) -> Result<Rc<BotBox>, SwiftBotsError> {
        let core = self.core;
        let mut chat_core = self.chat_core;
        let mut tg_core = self.tg_core;
        tg_core.allowed_updates = self.allowed_updates;
        tg_core.skip_updates = self.skip_updates;
        let name = core.name.clone();
        chat_core.error_message = self.error_message.unwrap_or(chat_core.error_message);
        chat_core.refuse_message = self.refuse_message.unwrap_or(chat_core.refuse_message);
        chat_core.unknown_message = self.unknown_message.unwrap_or(chat_core.unknown_message);
        debug!("Building bot: {}", name);
        let tg_core = Arc::new(tg_core);
        let listener_entry = standard_listener(tg_core.clone());
        let sender_entry = standard_sender(tg_core.clone());
        let chat_context_template = chat_core.make_chat_context(sender_entry.clone());
        let token_trie = build_token_trie(chat_core.message_handlers)
            .map_err(|e| SwiftBotsError::InvalidCommand(name.to_string(), e.to_string()))?;
        let handler_entry = core
            .handler_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
        let base_handler = BaseHandler::<Request<Json>> { bot_entry: handler_entry };
        let service = box_bot_service(ServiceBuilder::new()
            .layer(DeconstructTgMessageLayer{})
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

    pub fn build_oneshot(self) -> Result<OneshotBot<Request<Json>>, SwiftBotsError> {
        trace!("TelegramBot:build_oneshot");
        let core = self.core;
        let mut chat_core = self.chat_core;
        let mut tg_core = self.tg_core;
        tg_core.allowed_updates = self.allowed_updates;
        tg_core.skip_updates = self.skip_updates;
        let name = core.name.clone();
        chat_core.error_message = self.error_message.unwrap_or(chat_core.error_message);
        chat_core.refuse_message = self.refuse_message.unwrap_or(chat_core.refuse_message);
        chat_core.unknown_message = self.unknown_message.unwrap_or(chat_core.unknown_message);
        debug!("Building bot: {}", name);
        let tg_core = Arc::new(tg_core);
        let sender_entry = standard_sender(tg_core.clone());
        let chat_context_template = chat_core.make_chat_context(sender_entry.clone());
        let token_trie = build_token_trie(chat_core.message_handlers)
            .map_err(|e| SwiftBotsError::InvalidCommand(name.to_string(), e.to_string()))?;
        let handler_entry = core
            .handler_entry
            .ok_or_else(|| SwiftBotsError::BotHasNoHandler(name.to_string()))?;
        let base_handler = BaseHandler::<Request<Json>> { bot_entry: handler_entry };
        let service = box_bot_service(ServiceBuilder::new()
            .layer(DeconstructTgMessageLayer{})
            .layer(ChatContextLayer{ctx_template: chat_context_template})
            .layer(RoutingLayer{trie: Arc::new(token_trie)})
            .service(EntryService { inner: base_handler })
        );
        Ok(OneshotBot {
            name: name,
            service: service
        })
    }
}

pub struct TelegramCore {
    pub token: String,
    pub http_client: reqwest::Client,
    pub allowed_updates: Vec<String>,
    pub skip_updates: bool,
}

impl TelegramCore {
    pub async fn fetch(
        &self,
        method: &str,
        data: &Json,
        opts: FetchOptions,
    ) -> Result<Json, SwiftBotsError> {
        debug!("TG API fetching: {}", method);
        let url = format!("https://api.telegram.org/bot{}/{}", self.token, method);
        let mut req_builder = self.http_client
            .post(url)
            .json(data);
        if let Some(headers) = opts.headers {
            req_builder = req_builder.headers(headers);
        }
        if let Some(timeout) = opts.timeout {
            req_builder = req_builder.timeout(timeout);
        }

        let span = info_span!(
            "fetch",
            "method" = method,
            "retry_attempt" = 0u8,
            "response_code" = tracing::field::Empty,
        );
        self.try_fetch_request(req_builder, 3, opts.ignore_errors)
            .instrument(span)
            .await
    }

    pub async fn try_fetch_request(
        &self,
        req_builder: reqwest::RequestBuilder,
        max_retries: u8,
        ignore_errors: bool
    ) -> Result<Json, SwiftBotsError> {
        let mut retry_counter: u8 = 0;
        let mut current_builder = Some(req_builder);
        let mut last_error: Option<SwiftBotsError> = None;
        loop {
            tracing::Span::current().record(
                "retry_attempt",
                retry_counter,
            );
            let builder_for_this_run = if let Some(builder) = current_builder.take() {
                if let Some(cloned) = builder.try_clone() {
                    current_builder = Some(cloned);
                }
                builder
            } else {
                warn!("Request body was not clonable and first attempt failed");
                return Err(last_error.unwrap_or(SwiftBotsError::HttpError("Unknown TG Api error".to_string())));
            };
            let post_result = builder_for_this_run
                .send()
                .await
                .map_err(|e| SwiftBotsError::HttpError(e.to_string()));
            let response = match post_result {
                Ok(res) => res,
                Err(error) => {
                    if retry_counter >= max_retries {
                        warn!("TG API error with 200. Giving up on {} attempt", max_retries);
                        return Err(error);
                    }
                    last_error = Some(error);
                    debug!("Sleep for 2 second before retrying");
                    sleep(Duration::from_secs(2)).await;
                    info!("Fetch error. Retrying attempt: {}/{}", retry_counter+1, max_retries);
                    retry_counter += 1;
                    continue;
                }
            };
            let response_status = response.status().as_u16();
            debug!("TG API response status: {}", response_status);
            tracing::Span::current().record(
                "response_code",
                response_status,
            );
            match response_status {
                200 => {
                    let result = response
                        .json::<Json>()
                        .await
                        .map_err(|e| SwiftBotsError::HttpError(e.to_string()))?;
                    if !ignore_errors && !result["ok"].as_bool().unwrap_or_else(
                        || {
                            error!("TG API error: unexpected response: {}", result);
                            false
                        })
                    {
                        let (error_status, error) = Self::handle_tg_error(result).await;
                        match error_status {
                            TgApiErrorStatus::BadRequest => {
                                return Err(error);
                            },
                            TgApiErrorStatus::WaitAndRetry => {
                                if retry_counter >= max_retries {
                                    warn!("TG API error with 200. Giving up on {} attempt", max_retries);
                                    return Err(error);
                                }
                                last_error = Some(error);
                                debug!("Sleep for 5 second before retrying");
                                sleep(Duration::from_secs(5)).await;
                                info!("TG API error with 200. Retrying attempt: {}/{}", retry_counter+1, max_retries);
                            },
                            TgApiErrorStatus::ShouldShutdown => {
                                todo!("TG API critial error. Should shutdown");
                            }
                        }
                    }
                    else {
                        return Ok(result);
                    }
                },
                500..=599 => {
                    let error = SwiftBotsError::HttpError("TG API error: server error".to_string());
                    if retry_counter >= max_retries {
                        warn!("TG API server error {}. Giving up on {} attempt", response_status, max_retries);
                        return Err(error);
                    }
                    last_error = Some(error);
                    debug!("Sleep for 1 second before retrying");
                    sleep(Duration::from_secs(1)).await;
                    info!("TG API server error {}. Retrying attempt: {}/{}", response_status, retry_counter+1, max_retries);
                },
                _ => {
                    error!("TG API error: unexpected status code: {}", response_status);
                    return Err(SwiftBotsError::HttpError("TG API error: unexpected status code".to_string()));
                }
            };
            retry_counter += 1;
        }
    }

    pub async fn send_message(&self, ctx: SendFnContext){
        for msg in textwrap::wrap(ctx.message.as_str(), &*TEXT_WRAPPER) {
            let send = json!({"chat_id": ctx.recipient, "text": msg});
            self.fetch("sendMessage", &send, FetchOptions::default())
                .await
                .unwrap_or_else(|e| panic!("Failed to send message: {}", e));
        }
    }

    /*
    https://core.telegram.org/api/errors
     */
    pub async fn handle_tg_error(error: Json) -> (TgApiErrorStatus, SwiftBotsError) {
        let error_code = error["error_code"].as_i64().unwrap_or_else (|| { 
            error!("TG API error: missing or unexpected error code type: {}", error["error_code"]);
            400
        });
        let error_msg = error["description"].as_str().unwrap_or_else(|| {
            error!("TG API error: missing or unexpected error description: {}", error["description"]);
            "Unknown error"
        });
        let mut msg = format!("TG API error {}: {}", error_code, error_msg);
        match error_code {
            400 | 403 | 404 | 406 | 303 | 500..=599  => {
                error!(msg);
                return (TgApiErrorStatus::BadRequest, SwiftBotsError::HttpError(msg));
            },
            420 => {
                warn!(msg);
                return (TgApiErrorStatus::WaitAndRetry, SwiftBotsError::HttpError(msg));
            },
            401 => {
                error!(msg);
                return (TgApiErrorStatus::ShouldShutdown, SwiftBotsError::HttpError(msg));
            },
            409 => {
                msg = "Error code 409. Another telegram instance is working. Shutting down this instance".to_string();
                error!(msg);
                return (TgApiErrorStatus::ShouldShutdown, SwiftBotsError::HttpError(msg));
            }
            _ => {
                error!("Unknown TG API error: {}", error_code);
                return (TgApiErrorStatus::BadRequest, SwiftBotsError::HttpError(msg));
            }
        }
    }

    pub async fn get_updates(&self, tx: UnboundedSender<Request<Json>>) {
        const TIMEOUT: u16 = 1000;
        let mut data = json!({
            "timeout": TIMEOUT,
            "limit": 1,
            "allowed_updates": self.allowed_updates,
        });
        let opts = FetchOptions {
            timeout: Some(Duration::from_secs(TIMEOUT as u64 + 10)),
            ..Default::default()
        };
        if self.skip_updates {
            let offset = self.skip_updates().await.unwrap_or_else(|_| panic!("Cannot fetch"));
            debug!("Updates offset {}", offset);
            data.as_object_mut().unwrap().insert("offset".to_string(), offset.into());
        }
        info!("Start fetching updates");
        loop {
            let result = self.fetch("getUpdates", &data, opts.clone()).await;
            match result {
                Ok(mut answer) => {
                    if let Json::Array(updates) = answer["result"].take() {
                        if !updates.is_empty() {
                            let new_offset = 1 + updates[0]["update_id"].as_i64().unwrap_or_else(|| panic!("TG API error: unexpected update_id type: {}", answer));
                            data.as_object_mut().unwrap().insert("offset".to_string(), new_offset.into());
                            for update in updates {
                                let req = Request::builder()
                                    .extension(UpdateMeta { update: update })
                                    .body(json!({}));
                                if let Ok(req) = req {
                                    tx.send(req).unwrap();
                                }
                                else {
                                    error!("Failed to process update: {}", answer);
                                }
                            }
                        }
                    }
                    else {
                        panic!("TG API error: unexpected result type: {}", answer);
                    }
                },
                Err(error) => {
                    error!("Error while fetching updates: {}", error);
                }
            }
            debug!("Fetching next update");
        }
    }

    pub async fn skip_updates(&self) -> Result<i64, SwiftBotsError> {
        debug!("Skipping updates");
        let data = json!({
            "timeout": 0,
            "limit": 1,
            "offset": -1,
        });
        let ans = self.fetch("getUpdates", &data, FetchOptions::default()).await?;
        let result = ans["result"].as_array().unwrap_or_else(|| panic!("TG API error: unexpected result type: {}", ans));
        if result.is_empty() {
            return Ok(-1);
        }
        let update_id = result[0]["update_id"].as_i64().unwrap_or_else(|| panic!("TG API error: unexpected update_id type: {}", result[0]["update_id"]));
        Ok(update_id + 1)
    }
}