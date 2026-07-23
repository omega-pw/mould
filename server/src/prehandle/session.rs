use crate::native_common;
use crate::native_common::cache::AsyncCache;
use crate::Context;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use headers::Cookie;
use hyper::body::Incoming;
use hyper::header::HeaderMap;
use hyper::header::HeaderValue;
use hyper::Request;
use log;
use native_common::utils::decrypt_by_base64;
use native_common::utils::new_rsa_pub_key;
use native_common::utils::sha1;
use native_common::utils::verify_by_pub_pri_key;
use openid::Bearer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::base62;
use tihu::client_id::ClientId;
use tihu::encoder;
use tihu::version_data;
use tihu::Id;
use tihu::SharedString;
use tihu_native::http::FromRequest;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;
use uuid::Uuid;

pub const SESSION_KEY: &str = "SESSION";
pub const SESSION_PREFIX: &'static str = "session-";

#[derive(Clone, Copy)]
pub struct SessionId(pub u128);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", base62::encode(&self.0.to_be_bytes()))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: Id,
    pub timeout: u8, //单位：分钟
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionV1 {
    id: u128,
    client_id: SharedString,
    renew_time: i64, //单位：秒
    user: Option<User>,
}

#[derive(Clone, Debug)]
pub enum Session {
    V1(SessionV1),
}

impl Session {
    pub fn new(id: u128, client_id: SharedString, renew_time: DateTime<Utc>) -> Session {
        let renew_time = renew_time.timestamp();
        Session::V1(SessionV1 {
            id: id,
            client_id: client_id,
            renew_time: renew_time, //单位：秒
            user: None,
        })
    }

    fn version(&self) -> u64 {
        match self {
            Session::V1(_) => 1,
        }
    }

    pub fn id(&self) -> SessionId {
        match self {
            Session::V1(session) => session.id(),
        }
    }

    pub fn client_id(&self) -> &SharedString {
        match self {
            Session::V1(session) => &session.client_id,
        }
    }

    pub fn user(&self) -> Option<&User> {
        match self {
            Session::V1(session) => session.user(),
        }
    }

    pub fn set_user(&mut self, user: User) {
        match self {
            Session::V1(session) => {
                session.user = Some(user);
            }
        }
    }

    pub fn remove_user(&mut self) -> Option<User> {
        match self {
            Session::V1(session) => session.remove_user(),
        }
    }

    pub fn renew(&mut self, renew_time: DateTime<Utc>) {
        match self {
            Session::V1(session) => session.renew(renew_time),
        }
    }

    pub fn time_to_renew(&self) -> bool {
        match self {
            Session::V1(session) => session.time_to_renew(),
        }
    }

    pub fn encode(&self, signer: &SessionSigner) -> Result<String, SharedString> {
        match self {
            Session::V1(session) => {
                let data = serde_json::to_vec(&session).map_err(|err| {
                    log::error!("序列化会话数据失败: {}", err);
                    return SharedString::from_static("序列化会话数据失败!");
                })?;
                let signature = signer.sign(&data);
                let chunk = encoder::encode_chunks(&[&data, &signature], None);
                return version_data::encode(self.version(), &chunk);
            }
        }
    }

    pub fn try_decode(session_data: &str, signer: &SessionSigner) -> Result<Session, SharedString> {
        let (version, chunk) = version_data::try_decode(session_data)?;
        match version {
            1 => {
                let (data, signature) = encoder::decode_chunks::<2>(chunk)?;
                if signer.check_signature(&data, &signature) {
                    let session: SessionV1 = serde_json::from_slice(&data).map_err(|err| {
                        log::error!("反序列化会话数据失败: {}", err);
                        return SharedString::from_static("反序列化会话数据失败!");
                    })?;
                    return Ok(Session::V1(session));
                } else {
                    return Err(SharedString::from_static("会话数据签名不正确!"));
                }
            }
            _ => {
                return Err(SharedString::from_static("未知的会话数据版本!"));
            }
        }
    }
}

pub struct SessionSigner {
    pub sign_secret: Arc<Vec<u8>>,
}

impl SessionSigner {
    pub fn sign(&self, data: &[u8]) -> [u8; 20] {
        let plain: Vec<u8> = [data, &self.sign_secret].concat();
        let signature = sha1(&plain);
        return signature;
    }
    pub fn check_signature(&self, data: &[u8], signature: &[u8]) -> bool {
        if 20 != signature.len() {
            return false;
        }
        let actual_signature = self.sign(data);
        return &actual_signature == signature;
    }
}

impl SessionV1 {
    pub fn new(id: u128, client_id: SharedString, renew_time: DateTime<Utc>) -> SessionV1 {
        let renew_time = renew_time.timestamp();
        SessionV1 {
            id: id,
            client_id: client_id,
            renew_time: renew_time, //单位：秒
            user: None,
        }
    }

    pub fn id(&self) -> SessionId {
        return SessionId(self.id);
    }

    pub fn user(&self) -> Option<&User> {
        return self.user.as_ref();
    }

    pub fn remove_user(&mut self) -> Option<User> {
        return self.user.take();
    }

    pub fn remove_user_if_expired(&mut self) -> Option<User> {
        match self.user.as_ref() {
            Some(user) => {
                let expire_time = DateTime::from_timestamp(self.renew_time, 0).unwrap()
                    + Duration::minutes(user.timeout as i64);
                let curr_time = Utc::now();
                if curr_time >= expire_time {
                    return self.remove_user();
                } else {
                    return None;
                }
            }
            None => return None,
        };
    }

    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    pub fn renew(&mut self, renew_time: DateTime<Utc>) {
        self.renew_time = renew_time.timestamp();
    }

    pub fn time_to_renew(&self) -> bool {
        let curr_time = Utc::now();
        let diff_time = curr_time - DateTime::from_timestamp(self.renew_time, 0).unwrap();
        return diff_time >= Duration::seconds(60);
    }
}

#[derive(Clone, Debug)]
pub struct SessionState {
    pub session: Session,
    pub is_new: bool,
}

#[async_trait]
impl FromRequest<Arc<Context>, ErrNo> for SessionState {
    async fn try_extract(
        context: &Arc<Context>,
        request: &Request<Incoming>,
        remote_addr: SocketAddr,
        request_data: &mut RequestData,
    ) -> Result<Self, ErrNo> {
        let signature_result = request_data
            .try_get::<SignatureResult, Arc<Context>, ErrNo>(context, &request, remote_addr)
            .await?;
        let client_id = signature_result.client_id.client_id().clone();
        let cookie = request_data
            .try_get::<Option<Cookie>, Arc<Context>, ErrNo>(context, &request, remote_addr)
            .await?;
        let session_data = cookie
            .as_ref()
            .map(|cookies| cookies.get(SESSION_KEY))
            .flatten();
        let sign_secret = context.get_sign_secret().await?;
        let session_signer = SessionSigner {
            sign_secret: sign_secret.clone(),
        };
        let (session, is_new) = if let Some(session_data) = session_data {
            match Session::try_decode(session_data, &session_signer) {
                Ok(session) => {
                    if session.client_id().as_ref() == client_id.as_ref() {
                        (session, false)
                    } else {
                        (
                            Session::new(Uuid::new_v4().as_u128(), client_id, Utc::now()),
                            true,
                        )
                    }
                }
                Err(err) => {
                    log::error!("解码会话数据失败: {}", err);
                    (
                        Session::new(Uuid::new_v4().as_u128(), client_id, Utc::now()),
                        true,
                    )
                }
            }
        } else {
            //cookie里没有sessionId
            (
                Session::new(Uuid::new_v4().as_u128(), client_id, Utc::now()),
                true,
            )
        };
        return Ok(SessionState {
            session: session,
            is_new: is_new,
        });
    }
}

pub struct SignatureResult {
    pub client_id: ClientId,
    pub body_hash: Vec<u8>,
}

#[async_trait]
impl<Ctx> FromRequest<Ctx, ErrNo> for SignatureResult {
    async fn try_extract(
        _context: &Ctx,
        request: &Request<Incoming>,
        _remote_addr: SocketAddr,
        _request_data: &mut RequestData,
    ) -> Result<Self, ErrNo> {
        let client_id_data = get_header(request.headers(), "X-Client-Data").ok_or_else(|| {
            return SharedString::from_static("请求没有X-Client-Data请求头！");
        })?;
        let body_hash = get_header(request.headers(), "X-Hash").ok_or_else(|| {
            return SharedString::from_static("请求没有X-Hash请求头！");
        })?;
        let body_hash = decrypt_by_base64(body_hash)?;
        let client_id =
            try_decode_client_id(client_id_data, request.uri().path().as_bytes(), &body_hash)?;
        return Ok(SignatureResult {
            client_id,
            body_hash,
        });
    }
}

pub fn get_header<K>(headers: &HeaderMap<HeaderValue>, header_name: K) -> Option<&str>
where
    K: hyper::header::AsHeaderName,
{
    match headers.get(header_name) {
        Some(val) => match val.to_str() {
            Ok(v) => {
                return Some(v);
            }
            Err(err) => {
                log::error!("请求头格式异常: {:?}", err);
            }
        },
        None => (),
    }
    return None;
}

fn try_decode_client_id(
    client_id_data: &str,
    route: &[u8],
    body_hash: &[u8],
) -> Result<ClientId, SharedString> {
    return ClientId::try_decode(
        client_id_data,
        |rsa_pub_key: &str, client_id_data: &[u8], signature: &[u8]| {
            let rsa_pub_key = new_rsa_pub_key(rsa_pub_key)?;
            verify_by_pub_pri_key(
                &[route, body_hash, &client_id_data].concat(),
                &signature,
                &rsa_pub_key,
            )
            .map_err(|err| {
                log::error!("客户端身份数据签名不正确: {:?}", err);
                return SharedString::from_static("客户端身份数据签名不正确！");
            })?;
            return Ok(true);
        },
    );
}

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
pub enum AuthMethod {
    System,
    Oauth2(Oauth2Token),
    Openid(OpenidToken),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionInfo {
    pub auth_method: AuthMethod,
    pub user_id: Id,
    pub org_id: Option<Id>,
}

pub async fn get_session_data(
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
