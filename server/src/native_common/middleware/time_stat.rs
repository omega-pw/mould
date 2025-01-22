use crate::log;
use async_trait::async_trait;
use std::fmt::Display;
use std::marker::PhantomData;
use std::time::Instant;
use tihu::Handler;
use tihu::Middleware;

pub struct TimeStatHandler<H, In> {
    inner: H,
    phantom: PhantomData<In>,
}

#[async_trait]
impl<H, NS, D> Handler<(NS, D)> for TimeStatHandler<H, (NS, D)>
where
    H: Handler<(NS, D)>,
    H::Out: Send + Sync + 'static,
    NS: Display + Clone + Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    type Out = H::Out;
    async fn handle(&self, input: (NS, D)) -> Self::Out {
        let route = input.0.clone();
        let now = Instant::now();
        let output = self.inner.handle(input).await;
        let cost = now.elapsed().as_millis();
        if cost > 1000 {
            //大于1秒就警告
            log::warn!("time cost: {} {}ms", route, cost);
        } else {
            log::info!("time cost: {} {}ms", route, cost);
        }
        return output;
    }
}

impl<H, In> TimeStatHandler<H, In>
where
    H: Handler<In>,
    H::Out: Send + Sync + 'static,
{
    pub fn new(handler: H) -> Self {
        TimeStatHandler {
            inner: handler,
            phantom: PhantomData,
        }
    }
}

pub struct TimeStatMiddleware {}

impl<H, NS, D> Middleware<(NS, D), H> for TimeStatMiddleware
where
    H: Handler<(NS, D)>,
    H::Out: Send + Sync + 'static,
    NS: Display + Clone + Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    type Output = TimeStatHandler<H, (NS, D)>;

    fn transform(self, handler: H) -> Self::Output {
        TimeStatHandler::new(handler)
    }
}

impl TimeStatMiddleware {
    pub fn new() -> TimeStatMiddleware {
        TimeStatMiddleware {}
    }
}
