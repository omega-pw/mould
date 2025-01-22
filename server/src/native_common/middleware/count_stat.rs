use crate::log;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use tihu::Handler;
use tihu::Middleware;

pub struct CountStatHandler<E> {
    inner: E,
    count: AtomicU64,
}

#[async_trait]
impl<In, E> Handler<In> for CountStatHandler<E>
where
    E: Handler<In>,
    In: Send + Sync + 'static,
    E::Out: Send + Sync + 'static,
{
    type Out = E::Out;
    async fn handle(&self, input: In) -> Self::Out {
        let count = 1 + self.count.fetch_add(1, Ordering::Relaxed);
        if count > 10000 {
            //并发10000就警告
            log::warn!("current request count: {}", count);
        } else {
            log::info!("current request count: {}", count);
        }
        let output = self.inner.handle(input).await;
        self.count.fetch_sub(1, Ordering::Relaxed);
        return output;
    }
}

pub struct CountStatMiddleware {
    count: AtomicU64,
}

impl<In, E> Middleware<In, E> for CountStatMiddleware
where
    E: Handler<In>,
    In: Send + Sync + 'static,
    E::Out: Send + Sync + 'static,
{
    type Output = CountStatHandler<E>;

    fn transform(self, handler: E) -> Self::Output {
        CountStatHandler {
            inner: handler,
            count: self.count,
        }
    }
}

impl CountStatMiddleware {
    pub fn new() -> CountStatMiddleware {
        CountStatMiddleware {
            count: AtomicU64::new(0),
        }
    }
}
