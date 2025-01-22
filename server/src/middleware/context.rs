use crate::Context;
use crate::CONTEXT;
use async_trait::async_trait;
use std::sync::Arc;
use tihu::Handler;
use tihu::Middleware;

pub struct ContextHandler<E> {
    inner: E,
    context: Arc<Context>,
}

#[async_trait]
impl<In, E> Handler<In> for ContextHandler<E>
where
    In: Send + Sync + 'static,
    E: Handler<In>,
    E::Out: Send + Sync + 'static,
{
    type Out = E::Out;
    async fn handle(&self, input: In) -> Self::Out {
        CONTEXT
            .scope(self.context.clone(), self.inner.handle(input))
            .await
    }
}

pub struct ContextMiddleware {
    context: Arc<Context>,
}

impl<In, E> Middleware<In, E> for ContextMiddleware
where
    In: Send + Sync + 'static,
    E: Handler<In>,
    E::Out: Send + Sync + 'static,
{
    type Output = ContextHandler<E>;

    fn transform(self, handler: E) -> Self::Output {
        ContextHandler {
            inner: handler,
            context: self.context,
        }
    }
}

impl ContextMiddleware {
    pub fn new(context: Arc<Context>) -> ContextMiddleware {
        ContextMiddleware { context: context }
    }
}
