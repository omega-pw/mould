use super::session::SessionId;
use super::session::SessionState;
use crate::get_context;
use crate::json_response;
use crate::native_common;
use crate::native_common::cache::AsyncCache;
use crate::route::WHITE_LIST_NAMESPACE;
use crate::Context;
use async_trait::async_trait;
use bytes::Bytes;
use hyper::body::Incoming;
use hyper::{Request, Response};
use native_common::errno::gen_login_required;
use native_common::errno::result_to_json_resp;
use openid::Bearer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Api;
use tihu::Handler;
use tihu::Id;
use tihu::LightString;
use tihu::Middleware;
use tihu_native::http::Body;
use tihu_native::http::HttpData;
use tihu_native::http::HttpDataCache;
use tihu_native::ErrNo;

pub const SESSION_PREFIX: &'static str = "session-";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Oauth2Token {
    pub provider: String,
    pub access_token: String,
    pub openid: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OpenidToken {
    pub provider: String,
    pub bearer: Bearer,
    pub openid: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionInfo {
    pub auth_method: AuthMethod,
    pub user_id: Id,
    pub org_id: Option<Id>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AuthMethod {
    System,
    Oauth2(Oauth2Token),
    Openid(OpenidToken),
}

#[derive(Clone, Copy, Debug)]
pub enum ApiLevel {
    Guest,
    User,
}

#[async_trait]
impl HttpData for ApiLevel {
    async fn try_extract(
        request: &Request<Incoming>,
        _remote_addr: SocketAddr,
        _data_cache: &mut HttpDataCache,
    ) -> Result<Self, anyhow::Error> {
        return Ok(
            if WHITE_LIST_NAMESPACE
                .iter()
                .any(|namespace| request.uri().path().starts_with(namespace))
            {
                ApiLevel::Guest
            } else {
                ApiLevel::User
            },
        );
    }
}

#[derive(Clone, Copy)]
pub struct User {
    pub session_id: SessionId,
    pub user_id: Id,
    pub org_id: Option<Id>,
}

#[derive(Clone, Copy)]
pub struct Guest {
    pub session_id: SessionId,
}

#[derive(Clone)]
pub enum AuthLevel {
    Guest(Guest),
    User(User),
}

impl AuthLevel {
    pub fn session_id(&self) -> SessionId {
        match self {
            AuthLevel::Guest(guest) => guest.session_id,
            AuthLevel::User(user) => user.session_id,
        }
    }

    pub fn guest(&self) -> Guest {
        match self {
            AuthLevel::Guest(guest) => guest.clone(),
            AuthLevel::User(user) => Guest {
                session_id: user.session_id,
            },
        }
    }
}

#[async_trait]
impl HttpData for AuthLevel {
    async fn try_extract(
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        data_cache: &mut HttpDataCache,
    ) -> Result<Self, anyhow::Error> {
        let session_state = data_cache
            .try_get::<SessionState>(&request, remote_addr)
            .await?;
        let session_id = session_state.session.id();
        let context = get_context()?;
        let session_info = get_session_data(&context, &session_id.to_string()).await?;
        let auth_level = if let Some(session_info) = session_info {
            AuthLevel::User(User {
                session_id: session_id,
                user_id: session_info.user_id,
                org_id: session_info.org_id,
            })
        } else {
            AuthLevel::Guest(Guest {
                session_id: session_id,
            })
        };
        return Ok(auth_level);
    }
}

#[async_trait]
impl HttpData for Guest {
    async fn try_extract(
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        data_cache: &mut HttpDataCache,
    ) -> Result<Self, anyhow::Error> {
        let session_state = data_cache
            .try_get::<SessionState>(&request, remote_addr)
            .await?;
        let session_id = session_state.session.id();
        return Ok(Guest {
            session_id: session_id,
        });
    }
}

#[async_trait]
impl HttpData for User {
    async fn try_extract(
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        data_cache: &mut HttpDataCache,
    ) -> Result<Self, anyhow::Error> {
        let auth_level = data_cache
            .try_get::<AuthLevel>(&request, remote_addr)
            .await?;
        match auth_level {
            AuthLevel::Guest(_) => {
                return Err(LightString::from("用户未登录").into());
            }
            AuthLevel::User(user) => {
                return Ok(user.clone());
            }
        }
    }
}

async fn get_session_data(
    context: &Arc<Context>,
    session_id: &str,
) -> Result<Option<SessionInfo>, anyhow::Error> {
    let cache_mgr = context.get_cache_mgr().await?;
    let session_info = cache_mgr
        .get(&(String::from(SESSION_PREFIX) + &session_id).into_bytes())
        .await?;
    if let Some(session_info) = session_info {
        let session_info: SessionInfo = serde_json::from_slice(&session_info)?;
        return Ok(Some(session_info));
    } else {
        return Ok(None);
    }
}

pub type In = (Request<Incoming>, SocketAddr, HttpDataCache);
pub type Out = Result<Response<Body>, anyhow::Error>;

pub struct AuthHandler<E> {
    inner: E,
    context: Arc<Context>,
    white_list_namespace: Vec<LightString>,
}

#[async_trait]
impl<E> Handler<In> for AuthHandler<E>
where
    E: Handler<In, Out = Out>,
{
    type Out = E::Out;
    async fn handle(&self, (request, remote_addr, mut data_cache): In) -> Self::Out {
        let api_level = data_cache
            .try_get::<ApiLevel>(&request, remote_addr)
            .await?
            .clone();
        match api_level {
            ApiLevel::Guest => {
                //白名单，放行
            }
            ApiLevel::User => {
                let auth_level = data_cache
                    .try_get::<AuthLevel>(&request, remote_addr)
                    .await?;
                match auth_level {
                    AuthLevel::Guest(_) => {
                        //非白名单且未登录
                        return Ok(json_response(gen_login_required()));
                    }
                    AuthLevel::User(_) => {
                        //已登录，放行
                    }
                }
            }
        }
        self.inner.handle((request, remote_addr, data_cache)).await
    }
}

pub struct AuthMiddleware {
    context: Arc<Context>,
    white_list_namespace: Vec<LightString>,
}

impl<E> Middleware<In, E> for AuthMiddleware
where
    E: Handler<In, Out = Out>,
{
    type Output = AuthHandler<E>;

    fn transform(self, handler: E) -> Self::Output {
        AuthHandler {
            inner: handler,
            context: self.context,
            white_list_namespace: self.white_list_namespace,
        }
    }
}

impl AuthMiddleware {
    pub fn new(context: Arc<Context>, white_list_namespace: Vec<LightString>) -> AuthMiddleware {
        AuthMiddleware {
            context: context,
            white_list_namespace: white_list_namespace,
        }
    }
}

/**
 * 获取并校验请求
 */
pub fn get_and_validate_req<I>(_api: I, req: &[u8]) -> Result<I::Input, ErrNo>
where
    I: Api,
    I::Input: DeserializeOwned,
{
    let req = serde_json::from_slice(req).map_err(|err| {
        log::error!("请求参数格式错误: {:?}", err);
        ErrNo::ParamFormatError
    })?;
    I::validate_input(&req).map_err(|err| {
        log::error!("请求参数校验失败: {:?}", err);
        ErrNo::ParamInvalid(err)
    })?;
    return Ok(req);
}

/**
 * 调用api
 */
pub async fn try_call_api<F, I>(
    api: I,
    handler: impl Fn(AuthLevel, I::Input) -> F,
    auth_level: AuthLevel,
    req: &[u8],
) -> Result<I::Output, ErrNo>
where
    F: Future<Output = Result<I::Output, ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    let req = get_and_validate_req(api, req)?;
    return handler(auth_level, req).await;
}

/**
 * 调用api
 */
pub async fn call_api<F, I>(
    api: I,
    handler: impl Fn(AuthLevel, I::Input) -> F,
    auth_level: AuthLevel,
    req: &[u8],
) -> Bytes
where
    F: Future<Output = Result<I::Output, ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    result_to_json_resp(try_call_api(api, handler, auth_level, req).await).into()
}

/**
 * 调用开放api
 */
pub async fn try_call_guest_api<F, I>(
    api: I,
    handler: impl Fn(Guest, I::Input) -> F,
    guest: Guest,
    req: &[u8],
) -> Result<I::Output, ErrNo>
where
    F: Future<Output = Result<I::Output, ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    let req = get_and_validate_req(api, req)?;
    return handler(guest, req).await;
}

/**
 * 调用开放api
 */
pub async fn call_guest_api<F, I>(
    api: I,
    handler: impl Fn(Guest, I::Input) -> F,
    guest: Guest,
    req: &[u8],
) -> Bytes
where
    F: Future<Output = Result<I::Output, ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    result_to_json_resp(try_call_guest_api(api, handler, guest, req).await).into()
}

/**
 * 调用受控api
 */
pub async fn try_call_user_api<F, I>(
    api: I,
    handler: impl Fn(Id, User, I::Input) -> F,
    user: User,
    req: &[u8],
) -> Result<I::Output, ErrNo>
where
    F: Future<Output = Result<I::Output, tihu_native::ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    let org_id = user.org_id.ok_or_else(|| ErrNo::NotAllowed)?;
    let req = get_and_validate_req(api, req)?;
    return handler(org_id, user, req).await;
}

/**
 * 调用受控api
 */
pub async fn call_user_api<F, I>(
    api: I,
    handler: impl Fn(Id, User, I::Input) -> F,
    user: User,
    req: &[u8],
) -> Bytes
where
    F: Future<Output = Result<I::Output, tihu_native::ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    result_to_json_resp(try_call_user_api(api, handler, user, req).await).into()
}
