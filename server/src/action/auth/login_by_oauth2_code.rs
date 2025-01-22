use super::cache_session_info;
use crate::context::RPC_TIMEOUT;
use crate::get_context;
use crate::middleware::auth::AuthMethod;
use crate::middleware::auth::Guest;
use crate::middleware::auth::Oauth2Token;
use crate::middleware::auth::SessionInfo;
use crate::model::external_user::enums::ProviderType;
use crate::model::external_user::ExternalUser;
use crate::model::external_user::ExternalUserOpt;
use crate::model::external_user::ExternalUserProperty;
use crate::model::organization::Organization;
use crate::model::user::enums::UserSource;
use crate::model::user::User;
use crate::model::user::UserOpt;
use crate::model::user::UserProperty;
use crate::sdk;
use crate::service::base::ExternalUserBaseService;
use crate::service::base::OrganizationBaseService;
use crate::service::base::UserBaseService;
use chrono::Utc;
use form_urlencoded::Serializer;
use oauth2::AuthorizationCode;
use oauth2::PkceCodeVerifier;
use oauth2::TokenResponse;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::User as SdkUser;
use sdk::auth::login_by_oauth2_code::LoginByOauth2CodeReq;
use sdk::auth::login_by_oauth2_code::LoginByOauth2CodeResp;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

#[derive(Serialize, Deserialize, Debug)]
pub struct WechatUser {
    openid: String,
    nickname: String,
    sex: f64,
    province: String,
    city: String,
    country: String,
    headimgurl: Option<String>,
    privilege: Vec<String>,
    unionid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubUser {
    id: i64,
    name: String,
    avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WechatTokenResp {
    access_token: String,
    expires_in: f64,
    refresh_token: String,
    openid: String,
    scope: String,
    unionid: String,
}

pub async fn login_by_oauth2_code(
    guest: Guest,
    login_by_oauth2_code_req: LoginByOauth2CodeReq,
) -> Result<LoginByOauth2CodeResp, ErrNo> {
    let LoginByOauth2CodeReq {
        provider,
        code,
        pkce_verifier,
    } = login_by_oauth2_code_req;
    let context = get_context()?;
    let (oauth2_client, oauth2_server) = context.get_oauth2_client(&provider)?;
    let (session_info, curr_user) = if "wechat" == provider {
        let token_url = &oauth2_server.token_url;
        let app_id = &oauth2_server.client_id;
        let secret = &oauth2_server.client_secret;
        let query: String = Serializer::new(String::new())
            .append_pair("appid", app_id)
            .append_pair("secret", secret)
            .append_pair("code", &code)
            .append_pair("grant_type", "authorization_code")
            .finish();
        let token_url = format!("{token_url}?{query}");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(RPC_TIMEOUT))
            .build()
            .map_err(|err| ErrNo::Other(err.into()))?;
        let resp = client
            .get(token_url)
            .header("User-Agent", "reqwest")
            .send()
            .await
            .map_err(|err| ErrNo::ApiError(err.into()))?;
        let token_resp: WechatTokenResp = resp
            .json()
            .await
            .map_err(|err| ErrNo::ApiError(err.into()))?;
        let query: String = Serializer::new(String::new())
            .append_pair("access_token", &token_resp.access_token)
            .append_pair("openid", &token_resp.openid)
            .finish();
        let userinfo_url = format!("https://api.weixin.qq.com/sns/userinfo?{query}");
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
        let mut client = context.get_db_client().await?;
        let transaction = client.transaction().await.map_err(open_transaction_error)?;
        let user_base_service = UserBaseService::new(&transaction);
        let external_user_base_service = ExternalUserBaseService::new(&transaction);
        let external_user_opt = external_user_base_service
            .query_external_user_one(&ExternalUserOpt {
                provider_type: Some(ProviderType::Oauth2),
                provider: Some(provider.clone()),
                openid: Some(token_resp.openid.clone()),
                ..ExternalUserOpt::empty()
            })
            .await?;
        let user_pair_opt = if let Some(external_user) = external_user_opt {
            let user_opt = user_base_service.read_user(external_user.id).await?;
            if let Some(user) = user_opt {
                Some((user, external_user))
            } else {
                None
            }
        } else {
            None
        };
        let curr_time = Utc::now();
        let (session_info, curr_user) = if let Some((user, external_user)) = user_pair_opt {
            let user_id = user.id;
            let org_id = user.org_id;
            let changes: Vec<UserProperty> = vec![
                UserProperty::Name(user_info.nickname),
                UserProperty::AvatarUrl(user_info.headimgurl),
                UserProperty::LastModifiedTime(curr_time),
            ];
            user_base_service.update_user(user_id, &changes).await?;
            let changes: Vec<ExternalUserProperty> = vec![
                ExternalUserProperty::Detail(Some(user_detail.clone())),
                ExternalUserProperty::LastModifiedTime(curr_time),
            ];
            external_user_base_service
                .update_external_user(user_id, &changes)
                .await?;
            (
                SessionInfo {
                    auth_method: AuthMethod::Oauth2(Oauth2Token {
                        provider: provider.clone(),
                        access_token: token_resp.access_token,
                        openid: external_user.openid.clone(),
                    }),
                    user_id: user_id,
                    org_id: org_id,
                },
                SdkUser {
                    id: user_id,
                    org_id: org_id,
                    name: Some(user.name),
                    avatar_url: user.avatar_url,
                    auth_source: AuthSource::External {
                        provider_type: sdk::user::enums::ProviderType::Oauth2,
                        provider: provider,
                        openid: external_user.openid,
                        detail: user_detail,
                    },
                },
            )
        } else {
            let user_opt = user_base_service.query_user_one(&UserOpt::empty()).await?;
            let org_id = if user_opt.is_none() {
                //第一个用户
                let organization_base_service = OrganizationBaseService::new(&transaction);
                let org_id = context.new_id();
                organization_base_service
                    .insert_organization(&Organization {
                        id: org_id,
                        name: String::from("默认组织"),
                        created_time: curr_time,
                        last_modified_time: curr_time,
                    })
                    .await?;
                Some(org_id)
            } else {
                None
            };
            let user_id = context.new_id();
            let user = User {
                id: user_id,
                org_id: org_id,
                user_source: UserSource::External,
                name: user_info.nickname,
                avatar_url: user_info.headimgurl,
                created_time: curr_time,
                last_modified_time: curr_time,
            };
            let external_user = ExternalUser {
                id: user_id,
                provider_type: ProviderType::Oauth2,
                provider: provider.clone(),
                openid: token_resp.openid.clone(),
                detail: Some(user_detail.clone()),
                created_time: curr_time,
                last_modified_time: curr_time,
            };
            user_base_service.insert_user(&user).await?;
            external_user_base_service
                .insert_external_user(&external_user)
                .await?;
            (
                SessionInfo {
                    auth_method: AuthMethod::Oauth2(Oauth2Token {
                        provider: provider.clone(),
                        access_token: token_resp.access_token,
                        openid: token_resp.openid.clone(),
                    }),
                    user_id: user_id,
                    org_id: org_id,
                },
                SdkUser {
                    id: user_id,
                    org_id: org_id,
                    name: Some(user.name),
                    avatar_url: user.avatar_url,
                    auth_source: AuthSource::External {
                        provider_type: sdk::user::enums::ProviderType::Oauth2,
                        provider: provider,
                        openid: token_resp.openid,
                        detail: user_detail,
                    },
                },
            )
        };
        transaction
            .commit()
            .await
            .map_err(commit_transaction_error)?;
        (session_info, curr_user)
    } else if "github" == provider {
        let mut client = oauth2_client.exchange_code(AuthorizationCode::new(code));
        if oauth2_server.pkce {
            if let Some(pkce_verifier) = pkce_verifier {
                client = client.set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier));
            } else {
                return Err(ErrNo::CommonError(LightString::from_static(
                    "Parameter \"pkce_verifier\" required!",
                )));
            }
        }
        let token_result = client
            .request_async(async_http_client_with_timeout)
            .await
            .map_err(|err| ErrNo::Other(err.into()))?;
        let access_token = token_result.access_token().secret().clone();
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
        let openid = user_info.id.to_string();
        let mut client = context.get_db_client().await?;
        let transaction = client.transaction().await.map_err(open_transaction_error)?;
        let user_base_service = UserBaseService::new(&transaction);
        let external_user_base_service = ExternalUserBaseService::new(&transaction);
        let external_user_opt = external_user_base_service
            .query_external_user_one(&ExternalUserOpt {
                provider_type: Some(ProviderType::Oauth2),
                provider: Some(provider.clone()),
                openid: Some(openid.clone()),
                ..ExternalUserOpt::empty()
            })
            .await?;
        let user_pair_opt = if let Some(external_user) = external_user_opt {
            let user_opt = user_base_service.read_user(external_user.id).await?;
            if let Some(user) = user_opt {
                Some((user, external_user))
            } else {
                None
            }
        } else {
            None
        };
        let curr_time = Utc::now();
        let (session_info, curr_user) = if let Some((user, external_user)) = user_pair_opt {
            let user_id = user.id;
            let org_id = user.org_id;
            let changes: Vec<UserProperty> = vec![
                UserProperty::Name(user_info.name),
                UserProperty::AvatarUrl(user_info.avatar_url),
                UserProperty::LastModifiedTime(curr_time),
            ];
            user_base_service.update_user(user_id, &changes).await?;
            let changes: Vec<ExternalUserProperty> = vec![
                ExternalUserProperty::Detail(Some(user_detail.clone())),
                ExternalUserProperty::LastModifiedTime(curr_time),
            ];
            external_user_base_service
                .update_external_user(user_id, &changes)
                .await?;
            (
                SessionInfo {
                    auth_method: AuthMethod::Oauth2(Oauth2Token {
                        provider: provider.clone(),
                        access_token: access_token,
                        openid: external_user.openid.clone(),
                    }),
                    user_id: user_id,
                    org_id: org_id,
                },
                SdkUser {
                    id: user_id,
                    org_id: org_id,
                    name: Some(user.name),
                    avatar_url: user.avatar_url,
                    auth_source: AuthSource::External {
                        provider_type: sdk::user::enums::ProviderType::Oauth2,
                        provider: provider,
                        openid: external_user.openid,
                        detail: user_detail,
                    },
                },
            )
        } else {
            let user_opt = user_base_service.query_user_one(&UserOpt::empty()).await?;
            let org_id = if user_opt.is_none() {
                //第一个用户
                let organization_base_service = OrganizationBaseService::new(&transaction);
                let org_id = context.new_id();
                organization_base_service
                    .insert_organization(&Organization {
                        id: org_id,
                        name: String::from("默认组织"),
                        created_time: curr_time,
                        last_modified_time: curr_time,
                    })
                    .await?;
                Some(org_id)
            } else {
                None
            };
            let user_id = context.new_id();
            let user = User {
                id: user_id,
                org_id: org_id,
                user_source: UserSource::External,
                name: user_info.name,
                avatar_url: user_info.avatar_url,
                created_time: curr_time,
                last_modified_time: curr_time,
            };
            let external_user = ExternalUser {
                id: user_id,
                provider_type: ProviderType::Oauth2,
                provider: provider.clone(),
                openid: openid.clone(),
                detail: Some(user_detail.clone()),
                created_time: curr_time,
                last_modified_time: curr_time,
            };
            user_base_service.insert_user(&user).await?;
            external_user_base_service
                .insert_external_user(&external_user)
                .await?;
            (
                SessionInfo {
                    auth_method: AuthMethod::Oauth2(Oauth2Token {
                        provider: provider.clone(),
                        access_token: access_token,
                        openid: openid.clone(),
                    }),
                    user_id: user_id,
                    org_id: org_id,
                },
                SdkUser {
                    id: user_id,
                    org_id: org_id,
                    name: Some(user.name),
                    avatar_url: user.avatar_url,
                    auth_source: AuthSource::External {
                        provider_type: sdk::user::enums::ProviderType::Oauth2,
                        provider: provider,
                        openid: openid,
                        detail: user_detail,
                    },
                },
            )
        };
        transaction
            .commit()
            .await
            .map_err(commit_transaction_error)?;
        (session_info, curr_user)
    } else {
        return Err(ErrNo::CommonError(LightString::from(format!(
            "Unsupported oauth2 provider \"{}\"!",
            provider
        ))));
    };
    cache_session_info(guest.session_id, &session_info).await?;
    return Ok(Some(curr_user));
}

async fn async_http_client_with_timeout(
    request: oauth2::HttpRequest,
) -> Result<oauth2::HttpResponse, oauth2::reqwest::Error<reqwest::Error>> {
    let client = {
        let builder = reqwest::Client::builder();

        // Following redirects opens the client up to SSRF vulnerabilities.
        // but this is not possible to prevent on wasm targets
        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder.redirect(reqwest::redirect::Policy::none());

        builder
            .timeout(Duration::from_secs(RPC_TIMEOUT))
            .build()
            .map_err(oauth2::reqwest::Error::Reqwest)?
    };

    let mut request_builder = client
        .request(request.method, request.url.as_str())
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder
        .build()
        .map_err(oauth2::reqwest::Error::Reqwest)?;

    let response = client
        .execute(request)
        .await
        .map_err(oauth2::reqwest::Error::Reqwest)?;

    let status_code = response.status();
    let headers = response.headers().to_owned();
    let chunks = response
        .bytes()
        .await
        .map_err(oauth2::reqwest::Error::Reqwest)?;
    Ok(oauth2::HttpResponse {
        status_code,
        headers,
        body: chunks.to_vec(),
    })
}
