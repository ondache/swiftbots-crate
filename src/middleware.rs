use std::sync::Arc;
use crate::context::{Context, HandlerBackContext, MiddlewareContext};
use crate::types::{CallNextMiddleware, Middleware};


pub fn from_fn<F, Fut>(f: F) -> Middleware
    where
        F: Fn(MiddlewareContext, CallNextMiddleware) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerBackContext> + Send + 'static
{
    Arc::new(move |ctx, next| {
        Box::pin(f(ctx, next))
    })
}

pub fn compose_middlewares(middlewares: Vec<Middleware>) -> CallNextMiddleware {
    let mut next_layer: CallNextMiddleware = Arc::new(|_| Box::pin(async { HandlerBackContext{} }));
    for middleware in middlewares.into_iter().rev() {
        let cur_layer = middleware;
        next_layer = Arc::new(move |ctx| {
            cur_layer(ctx, next_layer.clone())
        })
    }
    next_layer
}

pub async fn execute_handler(ctx: MiddlewareContext, _: CallNextMiddleware) -> HandlerBackContext {
    let request = ctx.request.unwrap_or_else(|| {
        panic!("Request is not set in middleware context");
    });
    let user_ctx = Context { message: request.message };
    (ctx.bot_box.handler)(user_ctx).await;
    HandlerBackContext{}
}
