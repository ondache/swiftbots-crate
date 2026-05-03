use std::future::Future;
use crate::types::BoxFuture;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;
use tower::{BoxError, Service};


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
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: TRequest) -> Self::Future {
        let entry = self.bot_entry.clone();
        Box::pin(async move {
            entry(req).await;
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
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: TRequest) -> Self::Future {
        let fut = self.inner.call(req);
        Box::pin(async move {
            fut.await.map_err(Into::into)
        })
    }
}