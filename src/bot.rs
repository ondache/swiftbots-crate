use std::sync::Arc;
use crate::context::{Context, Request};
use std::future::Future;
use crate::types::{ListenerFunction, HandlerFunction, Middleware, CallNextMiddleware};
use crate::middleware::{execute_handler, from_fn, compose_middlewares, trace_middleware, collect_user_context_middleware};
use tokio::sync::mpsc;
use tracing::debug;

pub struct Bot {
    pub name: String,
    pub run_at_startup: bool,
    listener_entry: Option<ListenerFunction>,
    handler_entry: Option<HandlerFunction>,
    middlewares: Option<Vec<Middleware>>,
}

impl Bot {
    pub fn new(name: String) -> Self {
        Bot {
            name,
            run_at_startup: true,
            listener_entry: None,
            handler_entry: None,
            middlewares: None,
        }
    }

    pub fn listener<F, Fut>(mut self, listener_func: F) -> Self
        where
            F: Fn(mpsc::Sender<Request>) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static
    {
        self.listener_entry = Some(Arc::new(move |tx| {
            Box::pin(listener_func(tx))
        }));
        self
    }

    pub fn handler<F, Fut>(mut self, handler_func: F) -> Self
        where
            F: Fn(Context) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static
    {
        self.handler_entry = Some(Arc::new(move |ctx| {
            Box::pin(handler_func(ctx))
        }));
        self
    }

    pub fn build(mut self) -> BotBox {
        debug!("Building bot: {}", self.name);
        self.configure_middlewares();
        let entry = compose_middlewares(self.middlewares.unwrap_or_else(|| {
            let msg = format!("Bot {} has no middlewares set", self.name);
            panic!("{}", msg);
        }));
        BotBox {
            listener: self.listener_entry.unwrap_or_else(|| {
                let msg = format!("Bot {} has no listener", self.name);
                panic!("{}", msg);
            }),
            handler: self.handler_entry.unwrap_or_else(|| {
                let msg = format!("Bot {} has no handler", self.name);
                panic!("{}", msg);
            }),
            enabled: self.run_at_startup,
            entry,
            bot: Bot {
                name: self.name,
                run_at_startup: self.run_at_startup,
                listener_entry: None,
                handler_entry: None,
                middlewares: None,
            },
        }
    }

    fn configure_middlewares(&mut self) {
        debug!("Configuring middlewares");

        self.middlewares = Some(vec![
            from_fn(trace_middleware),
            from_fn(collect_user_context_middleware),
            from_fn(execute_handler),
        ])
    }
}

pub struct BotBox {
    pub bot: Bot,
    pub enabled: bool,
    pub listener: ListenerFunction,
    pub handler: HandlerFunction,
    pub entry: CallNextMiddleware,
}