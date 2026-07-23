use crate::log;
use async_trait::async_trait;
use hyper::body::Incoming;
use hyper::{Request, Response};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tihu::Handler;
use tihu::Middleware;
use tihu::SharedString;
use tihu_native::http::Body;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;

pub type In<Ctx> = (Ctx, Request<Incoming>, SocketAddr, RequestData);
pub type Out = Result<Response<Body>, ErrNo>;

pub struct TimeStatHandler<H, In> {
    inner: H,
    exclude_list: Arc<Vec<SharedString>>,
    phantom: PhantomData<In>,
}

#[async_trait]
impl<H, Ctx> Handler<In<Ctx>> for TimeStatHandler<H, In<Ctx>>
where
    H: Handler<In<Ctx>, Out = Out>,
    Ctx: Send + Sync + 'static,
{
    type Out = H::Out;
    async fn handle(&self, (context, request, remote_addr, request_data): In<Ctx>) -> Self::Out {
        let route = request.uri().path();
        if self
            .exclude_list
            .iter()
            .any(|namespace| route.starts_with(namespace.as_ref()))
        {
            return self
                .inner
                .handle((context, request, remote_addr, request_data))
                .await;
        } else {
            let route = route.to_string();
            let now = Instant::now();
            let output = self
                .inner
                .handle((context, request, remote_addr, request_data))
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
}

#[derive(Clone)]
pub struct TimeStatMiddleware {
    exclude_list: Arc<Vec<SharedString>>,
}

impl<H, Ctx> Middleware<In<Ctx>, H> for TimeStatMiddleware
where
    H: Handler<In<Ctx>, Out = Out>,
    Ctx: Send + Sync + 'static,
{
    type Output = TimeStatHandler<H, In<Ctx>>;

    fn transform(self, handler: H) -> Self::Output {
        TimeStatHandler {
            inner: handler,
            exclude_list: self.exclude_list,
            phantom: PhantomData,
        }
    }
}

impl TimeStatMiddleware {
    pub fn new(exclude_list: Option<Vec<SharedString>>) -> TimeStatMiddleware {
        TimeStatMiddleware {
            exclude_list: Arc::new(exclude_list.unwrap_or_default()),
        }
    }
}
