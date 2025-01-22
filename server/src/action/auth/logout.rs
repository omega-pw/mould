use crate::get_context;
use crate::middleware::auth::AuthMethod;
use crate::middleware::auth::Guest;
use crate::middleware::auth::SessionInfo;
use crate::middleware::auth::SESSION_PREFIX;
use crate::native_common;
use crate::sdk;
use crate::Context;
use form_urlencoded::Serializer;
use native_common::cache::AsyncCache;
use sdk::auth::logout::LogoutReq;
use sdk::auth::logout::LogoutResp;
use std::sync::Arc;
use tihu::LightString;
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

pub async fn get_logout_url(
    guest: Guest,
    get_logout_url_req: LogoutReq,
) -> Result<LogoutResp, ErrNo> {
    let LogoutReq { redirect_uri } = get_logout_url_req;
    let context = get_context()?;
    let session_id = guest.session_id.to_string();
    let cache_mgr = context.get_cache_mgr().await?;
    let session_info = get_session_data(&context, &session_id.to_string()).await?;
    if let Some(session_info) = session_info {
        let token_key = (String::from(SESSION_PREFIX) + &session_id).into_bytes();
        cache_mgr.remove(&token_key).await?;
        match session_info.auth_method {
            AuthMethod::Openid(token_info) => {
                if let Some(id_token) = token_info.bearer.id_token {
                    let (openid_client, openid_server) =
                        context.get_openid_client(&token_info.provider)?;
                    let end_session_endpoint = openid_client
                        .config()
                        .end_session_endpoint
                        .as_ref()
                        .map(|end_session_endpoint| end_session_endpoint.to_string())
                        .ok_or_else(|| {
                            ErrNo::CommonError(LightString::from_static(
                                "No end_session_endpoint found!",
                            ))
                        })?;
                    let redirect_uri = redirect_uri
                        .unwrap_or_else(|| format!("{}/login", context.config.public_path));
                    let query: String = Serializer::new(String::new())
                        .append_pair("client_id", &openid_server.client_id)
                        .append_pair("id_token_hint", &id_token)
                        .append_pair("post_logout_redirect_uri", &redirect_uri)
                        .finish();
                    let redirect_uri = format!("{end_session_endpoint}?{query}");
                    return Ok(LogoutResp {
                        redirect_uri: Some(redirect_uri),
                    });
                }
            }
            _ => (),
        }
    }
    return Ok(LogoutResp { redirect_uri: None });
}
