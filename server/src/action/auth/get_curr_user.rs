use crate::context::Context;
use crate::context::RPC_TIMEOUT;
use crate::get_context;
use crate::middleware::auth::AuthMethod;
use crate::middleware::auth::Guest;
use crate::middleware::auth::SessionInfo;
use crate::middleware::auth::SESSION_PREFIX;
use crate::native_common;
use crate::sdk;
use crate::service::base::SystemUserBaseService;
use crate::service::base::UserBaseService;
use form_urlencoded::Serializer;
use native_common::cache::AsyncCache;
use openid::Token;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::GetCurrUserReq;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::get_curr_user::User;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubUser {
    id: i64,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WechatUser {
    openid: String,
    nickname: Option<String>,
    sex: f64,
    province: String,
    city: String,
    country: String,
    headimgurl: Option<String>,
    privilege: Vec<String>,
    unionid: String,
}

pub async fn get_user_info(
    context: &Arc<Context>,
    session_info: &SessionInfo,
) -> Result<User, ErrNo> {
    match &session_info.auth_method {
        AuthMethod::System => {
            let user_id = session_info.user_id;
            let mut client = context.get_db_client().await?;
            let transaction = client.transaction().await.map_err(open_transaction_error)?;
            let user_base_service = UserBaseService::new(&transaction);
            let user_opt = user_base_service.read_user(user_id).await?;
            let user = user_opt.ok_or_else(|| -> ErrNo {
                ErrNo::CommonError(LightString::from_static("不存在此用户！"))
            })?;
            let system_user_base_service = SystemUserBaseService::new(&transaction);
            let system_user_opt = system_user_base_service.read_system_user(user_id).await?;
            let system_user = system_user_opt.ok_or_else(|| -> ErrNo {
                ErrNo::CommonError(LightString::from_static("不存在此用户！"))
            })?;
            return Ok(User {
                id: user_id,
                org_id: session_info.org_id,
                name: Some(user.name),
                avatar_url: user.avatar_url,
                auth_source: AuthSource::System {
                    email: system_user.email,
                    user_random_value: system_user.user_random_value,
                },
            });
        }
        AuthMethod::Openid(openid_token) => {
            let (openid_client, _) = context.get_openid_client(&openid_token.provider)?;
            let mut token = Token::from(openid_token.bearer.clone());
            let id_token = token
                .id_token
                .as_mut()
                .ok_or_else(|| ErrNo::LoginRequired)?;
            //必须进行decode_token操作，否则给request_userinfo操作用报错
            openid_client
                .decode_token(id_token)
                .map_err(|err| ErrNo::ApiError(err.into()))?;
            let userinfo = openid_client
                .request_userinfo(&token)
                .await
                .map_err(|err| ErrNo::ApiError(err.into()))?;
            let openid = userinfo.sub.clone().ok_or_else(|| {
                ErrNo::CommonError(LightString::from(
                    "No property \"sub\" found in user infomation!",
                ))
            })?;
            let user_detail = serde_json::to_string(&userinfo).map_err(ErrNo::SerializeError)?;
            return Ok(User {
                id: session_info.user_id,
                org_id: session_info.org_id,
                name: userinfo.nickname,
                avatar_url: userinfo.picture.map(|picture| picture.to_string()),
                auth_source: AuthSource::External {
                    provider_type: sdk::user::enums::ProviderType::Openid,
                    provider: openid_token.provider.clone(),
                    openid: openid,
                    detail: user_detail,
                },
            });
        }
        AuthMethod::Oauth2(oauth2_token) => {
            if "github" == oauth2_token.provider {
                let access_token = &oauth2_token.access_token;
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(RPC_TIMEOUT))
                    .build()
                    .map_err(|err| ErrNo::Other(err.into()))?;
                let resp = client
                    .get("https://api.github.com/user")
                    .header("User-Agent", "reqwest")
                    .header("Authorization", format!("Bearer {access_token}"))
                    .send()
                    .await
                    .map_err(|err| ErrNo::ApiError(err.into()))?;
                let user_detail: String = resp
                    .text()
                    .await
                    .map_err(|err| ErrNo::ApiError(err.into()))?;
                let user_info: GithubUser =
                    serde_json::from_str(&user_detail).map_err(ErrNo::DeserializeError)?;
                return Ok(User {
                    id: session_info.user_id,
                    org_id: session_info.org_id,
                    name: user_info.name,
                    avatar_url: user_info.avatar_url,
                    auth_source: AuthSource::External {
                        provider_type: sdk::user::enums::ProviderType::Oauth2,
                        provider: oauth2_token.provider.clone(),
                        openid: user_info.id.to_string(),
                        detail: user_detail,
                    },
                });
            } else if "wechat" == oauth2_token.provider {
                let access_token = &oauth2_token.access_token;
                let openid = &oauth2_token.openid;
                let query: String = Serializer::new(String::new())
                    .append_pair("access_token", access_token)
                    .append_pair("openid", openid)
                    .finish();
                let userinfo_url = format!("https://api.weixin.qq.com/sns/userinfo?{query}");
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(RPC_TIMEOUT))
                    .build()
                    .map_err(|err| ErrNo::Other(err.into()))?;
                let resp = client
                    .get(userinfo_url)
                    .header("User-Agent", "reqwest")
                    .send()
                    .await
                    .map_err(|err| ErrNo::ApiError(err.into()))?;
                let user_detail: String = resp
                    .text()
                    .await
                    .map_err(|err| ErrNo::ApiError(err.into()))?;
                let user_info: WechatUser =
                    serde_json::from_str(&user_detail).map_err(ErrNo::DeserializeError)?;
                return Ok(User {
                    id: session_info.user_id,
                    org_id: session_info.org_id,
                    name: user_info.nickname,
                    avatar_url: user_info.headimgurl,
                    auth_source: AuthSource::External {
                        provider_type: sdk::user::enums::ProviderType::Oauth2,
                        provider: oauth2_token.provider.clone(),
                        openid: user_info.openid,
                        detail: user_detail,
                    },
                });
            } else {
                return Err(ErrNo::CommonError(LightString::from(format!(
                    "Unsupported oauth2 provider \"{}\"!",
                    oauth2_token.provider
                ))));
            }
        }
    }
}

pub async fn get_curr_user(
    guest: Guest,
    _get_curr_user_req: GetCurrUserReq,
) -> Result<GetCurrUserResp, ErrNo> {
    let session_id = guest.session_id;
    let context = get_context()?;
    let session_info = get_session_data(&context, &session_id.to_string()).await?;
    if let Some(session_info) = session_info {
        let user_info = get_user_info(&context, &session_info).await?;
        return Ok(Some(user_info));
    } else {
        return Ok(None);
    }
}
