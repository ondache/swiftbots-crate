use std::pin::Pin;
use std::sync::Arc;
use http::Request;
use crate::basic::types::ListenerFunction;
use crate::chat::types::SenderFunction;
use crate::telegram::bot::TelegramCore;
use crate::telegram::types::Json;

pub fn standard_listener(tg_core: Arc<TelegramCore>) -> Arc<ListenerFunction<Request<Json>>> {
    Arc::new(move |tx| {
        let tg_core = tg_core.clone();
        Box::pin(async move {
            tg_core.get_updates(tx).await
        }) as Pin<Box<dyn Future<Output=()> + Send>>
    })
}

pub fn standard_sender(tg_core: Arc<TelegramCore>) -> Arc<SenderFunction> {
    Arc::new(move |ctx| {
        let tg_core = tg_core.clone();
        Box::pin(async move {
            tg_core.send_message(ctx).await
        }) as Pin<Box<dyn Future<Output=()> + Send>>
    })
}