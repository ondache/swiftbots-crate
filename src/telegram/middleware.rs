use std::task::{Context, Poll};
use http::Request;
use tower::{BoxError, Layer, Service};
use tracing::{info, trace};
use crate::chat::context::SenderMeta;
use crate::telegram::context::UpdateMeta;
use crate::telegram::types::Json;
use crate::types::BoxResultFuture;
use crate::types::MaybeSendFuture;

#[derive(Clone)]
pub struct DeconstructTgMessageMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Json>> for DeconstructTgMessageMiddleware<S>
where
    S: Service<Request<Json>> + Send,
    S::Error: Into<BoxError>,
    S::Future: MaybeSendFuture + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<Json>) -> Self::Future {
        trace!("DeconstructTgMessageMiddleware::call");
        let update = &req
            .extensions()
            .get::<UpdateMeta>()
            .expect("UpdateMeta not set in request extensions")
            .update;
        let message = update["message"].as_object().unwrap_or_else(|| {
            panic!("Update is not a message: {}", update);
        });
        let sender = message["from"]["id"].as_u64().unwrap_or_else(|| panic!("Unexpected format {}", update));
        let username: Option<&str> = message["from"]["username"].as_str();
        let text = message["text"].as_str().unwrap_or("");
        info!("Got message from {} ({})", sender, username.unwrap_or("unknown"));
        let feed_ctx = SenderMeta {
            sender: sender.to_string(),
            message: text.to_string(),
        };
        req.extensions_mut().insert::<SenderMeta>(feed_ctx);
        let fut = self.inner.call(req);
        Box::pin(async move {
            let awaited = fut.await.map_err(Into::into);
            trace!("DeconstructTgMessageMiddleware::finish");
            awaited
        })
    }
}

pub struct DeconstructTgMessageLayer {}

impl<S> Layer<S> for DeconstructTgMessageLayer {
    type Service = DeconstructTgMessageMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DeconstructTgMessageMiddleware { inner }
    }
}