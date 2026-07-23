use crate::prehandle::auth::Guest;
use crate::sdk;
use crate::Context;
use sdk::auth::get_openid_providers::GetOpenidProvidersReq;
use sdk::auth::get_openid_providers::GetOpenidProvidersResp;
use sdk::auth::get_openid_providers::OpenidProvider;
use std::sync::Arc;
use tihu_native::ErrNo;

pub async fn get_openid_providers(
    context: Arc<Context>,
    _guest: Guest,
    _get_curr_user_req: GetOpenidProvidersReq,
) -> Result<GetOpenidProvidersResp, ErrNo> {
    let openid_servers: Vec<_> = context
        .config
        .openid_servers
        .iter()
        .map(|(key, openid_server)| {
            return OpenidProvider {
                key: key.clone(),
                name: openid_server.name.clone(),
            };
        })
        .collect();
    return Ok(openid_servers);
}
