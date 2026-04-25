use std::sync::Arc;
use crate::context::{Context, MiddlewareContext};
use crate::types::{CallNextMiddleware, Middleware};
use tracing::{info_span, Instrument, debug};
use std::sync::atomic::{AtomicU64};
use crate::utils::generate_random_id;


static TRACE_ID_SEED: AtomicU64 = AtomicU64::new(1);
static CORRELATION_ID_SEED: AtomicU64 = AtomicU64::new(2);

pub fn from_fn<F, Fut>(f: F) -> Middleware
    where
        F: Fn(MiddlewareContext, CallNextMiddleware) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
{
    Arc::new(move |ctx, next| {
        Box::pin(f(ctx, next))
    })
}

pub fn compose_middlewares(middlewares: Vec<Middleware>) -> CallNextMiddleware {
    let mut next_layer: CallNextMiddleware = Arc::new(|_| Box::pin(async {}));
    for middleware in middlewares.into_iter().rev() {
        let cur_layer = middleware;
        next_layer = Arc::new(move |ctx| {
            cur_layer(ctx, next_layer.clone())
        })
    }
    next_layer
}

pub async fn trace_middleware(ctx: MiddlewareContext, next: CallNextMiddleware) {
    debug!("trace_middleware");
    let trace_id = format!("trace-{:016x}", generate_random_id(&TRACE_ID_SEED));
    let correlation_id = format!("trace-{:016x}", generate_random_id(&CORRELATION_ID_SEED));
    let span = info_span!(
        "request",
        "trace_id" = trace_id,
        "correlation_id" = correlation_id,
        "bot" = ctx.bot_box.name.clone(),
    );

    async move {
        next(ctx).await
    }
    .instrument(span)
    .await
}

pub async fn copy_user_context_middleware(mut ctx: MiddlewareContext, next: CallNextMiddleware) {
    debug!("collect_user_context_middleware");
    ctx.user_context = ctx.feed_context.clone();
    next(ctx).await
}

pub async fn execute_handler_middleware(ctx: MiddlewareContext, _: CallNextMiddleware) {
    debug!("execute_handler");
    let user_ctx: Context = Context { 
        req: ctx.user_context,
    };
    (ctx.bot_box.handler)(user_ctx).await
}
