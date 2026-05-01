use std::pin::Pin;
use std::sync::Arc;
use crate::context::{BasicRequest};
use crate::types::{HandlerFunction, Json};
use tracing::{info_span, Instrument, debug};
use std::sync::atomic::{AtomicU64};
use std::task::{Context, Poll};
use tower::BoxError;
use tower_layer::Layer;
use tower_service::Service;
use crate::utils::generate_random_id;


// static TRACE_ID_SEED: AtomicU64 = AtomicU64::new(1);
// static CORRELATION_ID_SEED: AtomicU64 = AtomicU64::new(2);
// 
// pub fn from_fn<F, Fut>(f: F) -> Middleware
//     where
//         F: Fn(MiddlewareContext, CallNextMiddleware) -> Fut + Send + Sync + 'static,
//         Fut: Future<Output = ()> + Send + 'static
// {
//     Arc::new(move |ctx, next| {
//         Box::pin(f(ctx, next))
//     })
// }
// 
// pub fn compose_middlewares(middlewares: Vec<Middleware>) -> CallNextMiddleware {
//     let mut next_layer: CallNextMiddleware = Arc::new(|_| Box::pin(async {}));
//     for middleware in middlewares.into_iter().rev() {
//         let cur_layer = middleware;
//         next_layer = Arc::new(move |ctx| {
//             cur_layer(ctx, next_layer.clone())
//         })
//     }
//     next_layer
// }
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
// pub async fn copy_user_context_middleware(mut ctx: MiddlewareContext, next: CallNextMiddleware) {
//     debug!("collect_user_context_middleware");
//     ctx.user_context = ctx.feed_context.clone();
//     next(ctx).await
// }

// pub async fn execute_handler_middleware(ctx: MiddlewareContext, _: CallNextMiddleware) {
//     debug!("execute_handler");
//     let user_ctx: BaseContext = BaseContext {
//         bot_box: ctx.bot_box.clone(),
//         req: ctx.user_context,
//     };
//     (ctx.bot_box.handler)(user_ctx).await
// }

// pub async fn fill_chat_context_middleware(mut ctx: MiddlewareContext, next: CallNextMiddleware) {
//     debug!("collect_user_context_middleware");
//     ctx.user_context = ctx.feed_context.clone();
//     next(ctx).await
// }

#[derive(Clone)]
pub struct BaseHandler <TRequest> {
    pub bot_entry: HandlerFunction<TRequest>,
}

impl<TRequest: Send + 'static> Service<TRequest> for BaseHandler<TRequest> {
    type Response = ();
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: TRequest) -> Self::Future {
        let entry = self.bot_entry.clone();
        Box::pin(async move {
            Ok(entry(req).await)
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


// pub struct LoggingMiddleware<S> {
//     inner: S,
// }
// 
// impl<S, Request> Service<Request> for LoggingMiddleware<S>
// where
//     S: Service<Request> + Send,
//     S::Error: Into<BoxError>,
//     S::Future: Send + 'static,
// {
//     type Response = S::Response;
//     type Error = BoxError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
// 
//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.inner.poll_ready(cx).map_err(Into::into)
//     }
// 
//     fn call(&mut self, req: Request) -> Self::Future {
//         let fut = self.inner.call(req);
//         Box::pin(async move {
//             let res = fut.await.map_err(Into::into);
//             println!("<-- [LoggingMiddleware] Finished request");
//             res
//         })
//     }
// }
// 
// pub struct LoggingLayer;
// 
// impl<S> Layer<S> for LoggingLayer {
//     type Service = LoggingMiddleware<S>;
// 
//     fn layer(&self, inner: S) -> Self::Service {
//         LoggingMiddleware { inner }
//     }
// }