use std::sync::Arc;
use http::Request;
use crate::chat::context::{ChatContext, RoutingMeta};
use crate::types::BoxFuture;

pub fn chat_handler_extractor<TBody: Send + Clone + 'static>(
    body_transform: impl Fn(Request<TBody>) -> Request<TBody> + Send + Sync + 'static,
) -> Arc<dyn Fn(Request<TBody>) -> BoxFuture<()> + Send + Sync> {
    Arc::new(move |mut req| {
        let chat_ctx: ChatContext;
        let routing_meta: RoutingMeta<TBody>;
        if let Some(ctx) = req.extensions().get::<ChatContext>() {
            chat_ctx = ctx.clone();
        } else { panic!("ChatContext not set in request extensions") }
        if let Some(rt) = req.extensions().get::<RoutingMeta<TBody>>() {
            routing_meta = rt.clone();
        } else { panic!("RoutingMeta not set in request extensions") }

        req = body_transform(req);
        Box::pin(async move {
            (routing_meta.handler)(req, chat_ctx).await
        })
    })
}
