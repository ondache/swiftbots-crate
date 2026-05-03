use tower::{BoxError, Service, Layer};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use http::Request;
use crate::chat::context::{ChatContext, RoutingMeta, SenderMeta};
use crate::chat::routing::{TokenTrie, search_matched_commands};

#[derive(Clone)]
pub struct RoutingMiddleware<S, TBody> {
    inner: S,
    trie: Arc<TokenTrie<TBody>>,
}

impl<S, TBody> Service<Request<TBody>> for RoutingMiddleware<S, TBody>
where
    S: Service<Request<TBody>> + Send,
    S::Error: Into<BoxError>,
    S::Future: Send + 'static,
    TBody: Send + 'static + Clone,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<TBody>) -> Self::Future {
        let chat_ctx: ChatContext;
        if let Some(ctx) = req.extensions().get::<ChatContext>() {
            chat_ctx = ctx.clone();
        }
        else {
            return Box::pin(
                async move {
                    Err(Into::into("ChatContext is not set in request extensions. Check middleware chain."))
                }
            )
        }
        let message: &str = chat_ctx.orig_message.as_str();
        let trie = self.trie.clone();
        let mut matches_queue = search_matched_commands(&trie, &message);
        loop {
            let handler_candidate = matches_queue.peek();
            if let Some(handler_candidate) = handler_candidate {
                let matched = handler_candidate.re_command.captures(message);
                match matched {
                    Some(captures) => {
                        let args = captures.get(1).map_or("", |m| m.as_str());
                        let routing_meta = RoutingMeta{
                            arguments: args.to_string(),
                            handler: handler_candidate.handler_entry.clone(),
                        };
                        req.extensions_mut().insert::<RoutingMeta<TBody>>(routing_meta);

                        let fut = self.inner.call(req);
                        return Box::pin(async move {
                            let res = fut.await.map_err(Into::into);
                            res
                        });
                    }
                    None => {
                        matches_queue.pop();
                    }
                }
            }
            else {
                return Box::pin(async move {
                    chat_ctx.unknown_command().await;
                    Err(Into::into("ChatContext is not set in request extensions. Check middleware chain."))
                });
            }
        }
    }
}

pub struct RoutingLayer <TBody> {
    pub trie: Arc<TokenTrie<TBody>>,
}

impl<S, TBody> Layer<S> for RoutingLayer<TBody> {
    type Service = RoutingMiddleware<S, TBody>;

    fn layer(&self, inner: S) -> Self::Service {
        RoutingMiddleware { inner, trie: self.trie.clone() }
    }
}


#[derive(Clone)]
pub struct ChatContextMiddleware<S> {
    inner: S,
    ctx_template: ChatContext,
}

impl<S, TBody> Service<Request<TBody>> for ChatContextMiddleware<S>
where
    S: Service<Request<TBody>> + Send,
    S::Error: Into<BoxError>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<TBody>) -> Self::Future {
        let mut chat_context = self.ctx_template.clone();
        if let Some(sender_context) = req.extensions().get::<SenderMeta>() {
            chat_context.sender = sender_context.sender.clone();
            chat_context.orig_message = sender_context.message.clone();
            req.extensions_mut().insert::<ChatContext>(chat_context);
        } else {
            return Box::pin(
                async move {
                    Err(Into::into("SenderContext not set in request extensions. Check middleware chain."))
                }
            )
        }
        let fut = self.inner.call(req);
        Box::pin(async move {
            let res = fut.await.map_err(Into::into);
            res
        })
    }
}

pub struct ChatContextLayer {
    pub ctx_template: ChatContext,
}

impl<S> Layer<S> for ChatContextLayer {
    type Service = ChatContextMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ChatContextMiddleware { inner, ctx_template: self.ctx_template.clone() }
    }
}