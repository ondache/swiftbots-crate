use crate::types::{BoxFuture, BoxResultFuture};
use std::task::{Context, Poll};
use std::sync::Arc;
use tower::{BoxError, Service};
use tracing::trace;
#[cfg(not(target_family = "wasm"))]
use tower::util::BoxCloneService;
use crate::types::MaybeSendFuture;
// static TRACE_ID_SEED: AtomicU64 = AtomicU64::new(1);
// static CORRELATION_ID_SEED: AtomicU64 = AtomicU64::new(2);
// 
// pub async fn trace_middleware(ctx: MiddlewareContext, next: CallNextMiddleware) {
//     debug!("trace_middleware");
//     let trace_id = format!("trace-{:016x}", generate_random_id(&TRACE_ID_SEED));
//     let correlation_id = format!("trace-{:016x}", generate_random_id(&CORRELATION_ID_SEED));
//     let span = info_span!(
//         "request",
//         "trace_id" = trace_id,
//         "correlation_id" = correlation_id,
//         "bot" = ctx.bot_box.name.clone(),
//     );
// 
//     async move {
//         next(ctx).await
//     }
//     .instrument(span)
//     .await
// }
// 


pub struct BaseHandler <TRequest> {
    pub bot_entry: Arc<dyn Fn(TRequest) -> BoxFuture<()> + Send + Sync>,
}

impl<TRequest> Clone for BaseHandler<TRequest> {
    fn clone(&self) -> Self {
        BaseHandler {
            bot_entry: self.bot_entry.clone(),
        }
    }
}

impl<TRequest: Send + Sync + 'static> Service<TRequest> for BaseHandler<TRequest> {
    type Response = ();
    type Error = BoxError;
    type Future = BoxResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: TRequest) -> Self::Future {
        trace!("BaseHandler::call");
        let entry = self.bot_entry.clone();
        Box::pin(async move {
            entry(req).await;
            trace!("BaseHandler::finish");
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct EntryService<S> {
    pub inner: S,
}

impl<S, TRequest> Service<TRequest> for EntryService<S>
where
    S: Service<TRequest> + Send,
    S::Error: Into<BoxError>,
    S::Future: MaybeSendFuture + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: TRequest) -> Self::Future {
        trace!("EntryService::call");
        let fut = self.inner.call(req);
        Box::pin(async move {
            fut.await.map_err(Into::into)
        })
    }
}

#[cfg(not(target_family = "wasm"))]
pub type BotService<TRequest> = BoxCloneService<TRequest, (), BoxError>;

#[cfg(not(target_family = "wasm"))]
pub fn box_bot_service<TRequest, S>(service: S) -> BotService<TRequest>
where
    S: Service<TRequest, Response = (), Error = BoxError> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    BoxCloneService::new(service)
}

#[cfg(target_family = "wasm")]
trait CloneBotService<TRequest>: Service<TRequest> {
    fn clone_box(&self) -> Box<dyn CloneBotService<TRequest, Response = Self::Response, Error = Self::Error, Future = Self::Future>>;
}

#[cfg(target_family = "wasm")]
pub struct BotService<TRequest>(
    Box<dyn CloneBotService<TRequest, Response = (), Error = BoxError, Future = BoxResultFuture<(), BoxError>>>,
);

#[cfg(target_family = "wasm")]
impl<TRequest> BotService<TRequest> {
    pub fn new<S>(service: S) -> Self
    where
        S: Service<TRequest, Response = (), Error = BoxError> + Clone + 'static,
        S::Future: 'static,
    {
        BotService(Box::new(BotServiceBoxed { inner: service }))
    }
}

#[cfg(target_family = "wasm")]
impl<TRequest> Service<TRequest> for BotService<TRequest> {
    type Response = ();
    type Error = BoxError;
    type Future = BoxResultFuture<(), BoxError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: TRequest) -> Self::Future {
        self.0.call(req)
    }
}

#[cfg(target_family = "wasm")]
impl<TRequest> Clone for BotService<TRequest> {
    fn clone(&self) -> Self {
        BotService(self.0.clone_box())
    }
}

#[cfg(target_family = "wasm")]
#[derive(Clone)]
struct BotServiceBoxed<S> {
    inner: S,
}

#[cfg(target_family = "wasm")]
impl<TRequest, S> Service<TRequest> for BotServiceBoxed<S>
where
    S: Service<TRequest> + Clone + 'static,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: TRequest) -> Self::Future {
        Box::pin(self.inner.call(req))
    }
}

#[cfg(target_family = "wasm")]
impl<TRequest, S> CloneBotService<TRequest> for BotServiceBoxed<S>
where
    S: Service<TRequest> + Clone + 'static,
    S::Future: 'static,
{
    fn clone_box(&self) -> Box<dyn CloneBotService<TRequest, Response = Self::Response, Error = Self::Error, Future = Self::Future>> {
        Box::new(self.clone())
    }
}

#[cfg(target_family = "wasm")]
pub fn box_bot_service<TRequest, S>(service: S) -> BotService<TRequest>
where
    S: Service<TRequest, Response = (), Error = BoxError> + Clone + 'static,
    S::Future: 'static,
{
    BotService::new(service)
}