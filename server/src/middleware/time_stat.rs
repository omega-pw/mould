use crate::log;
use async_trait::async_trait;
use hyper::body::Incoming;
use hyper::{Request, Response};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::time::Instant;
use tihu::Handler;
use tihu::Middleware;
use tihu_native::http::Body;
use tihu_native::http::RequestData;

pub type In = (Request<Incoming>, SocketAddr, RequestData);
pub type Out = Result<Response<Body>, anyhow::Error>;

pub struct TimeStatHandler<H, In> {
    inner: H,
    phantom: PhantomData<In>,
}

#[async_trait]
impl<H> Handler<In> for TimeStatHandler<H, In>
where
    H: Handler<In, Out = Out>,
{
    type Out = H::Out;
    async fn handle(&self, (request, remote_addr, request_data): In) -> Self::Out {
        let route = request.uri().path().to_string();
        let now = Instant::now();
        let output = self
            .inner
            .handle((request, remote_addr, request_data))
            .await;
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
    H: Handler<In, Out = Out>,
{
    pub fn new(handler: H) -> Self {
        TimeStatHandler {
            inner: handler,
            phantom: PhantomData,
        }
    }
}

pub struct TimeStatMiddleware {}

impl<H> Middleware<In, H> for TimeStatMiddleware
where
    H: Handler<In, Out = Out>,
{
    type Output = TimeStatHandler<H, In>;

    fn transform(self, handler: H) -> Self::Output {
        TimeStatHandler::new(handler)
    }
}

impl TimeStatMiddleware {
    pub fn new() -> TimeStatMiddleware {
        TimeStatMiddleware {}
    }
}
