use crate::native_common;
use crate::prehandle::session::SessionSigner;
use crate::prehandle::session::SessionState;
use crate::prehandle::session::SESSION_KEY;
use crate::Context;
use async_trait::async_trait;
use chrono::Utc;
use hyper::body::Incoming;
use hyper::header::HeaderValue;
use hyper::{Request, Response};
use log;
use native_common::cookie::format_cookie;
use native_common::cookie::CookieAttr;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Handler;
use tihu::Middleware;
use tihu::SharedString;
use tihu_native::http::Body;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;

pub type In = (Arc<Context>, Request<Incoming>, SocketAddr, RequestData);
pub type Out = Result<Response<Body>, ErrNo>;

pub struct SessionHandler<E> {
    inner: E,
    context: Arc<Context>,
}

#[async_trait]
impl<E> Handler<In> for SessionHandler<E>
where
    E: Handler<In, Out = Out>,
{
    type Out = Out;
    async fn handle(&self, (context, request, remote_addr, mut request_data): In) -> Self::Out {
        let mut session_state = request_data
            .try_get::<SessionState, Arc<Context>, ErrNo>(&context, &request, remote_addr)
            .await?
            .clone();
        let mut response = self
            .inner
            .handle((context, request, remote_addr, request_data))
            .await?;
        if session_state.session.time_to_renew() {
            let curr_time = Utc::now();
            session_state.session.renew(curr_time);
            session_state.is_new = true;
        }
        if session_state.is_new {
            let sign_secret = self.context.get_sign_secret().await?;
            let session_signer = SessionSigner {
                sign_secret: sign_secret.clone(),
            };
            let session = session_state
                .session
                .encode(&session_signer)
                .map_err(ErrNo::CommonError)?;
            if let Err(err) = set_cookie(&mut response, SESSION_KEY, &session) {
                log::error!("Write session to response failed, {:?}", err);
            }
        }
        return Ok(response);
    }
}

#[derive(Clone)]
pub struct SessionMiddleware {
    context: Arc<Context>,
}

impl<E> Middleware<In, E> for SessionMiddleware
where
    E: Handler<In, Out = Out>,
{
    type Output = SessionHandler<E>;

    fn transform(self, handler: E) -> Self::Output {
        SessionHandler {
            inner: handler,
            context: self.context,
        }
    }
}

impl SessionMiddleware {
    pub fn new(context: Arc<Context>) -> SessionMiddleware {
        SessionMiddleware { context: context }
    }
}

//设置cookie
pub fn set_cookie(resp: &mut Response<Body>, key: &str, value: &str) -> Result<(), SharedString> {
    let cookie = format_cookie(
        key,
        value,
        &CookieAttr {
            Path: Some(String::from("/")),
            HttpOnly: Some(()),
            ..CookieAttr::empty()
        },
    );
    let header_value = HeaderValue::from_str(&cookie).map_err(|err| {
        log::error!("生成响应头的值不符合规范: {:?}", err);
        return SharedString::from_static("生成响应头的值不符合规范");
    })?;
    resp.headers_mut().append("Set-Cookie", header_value);
    return Ok(());
}
