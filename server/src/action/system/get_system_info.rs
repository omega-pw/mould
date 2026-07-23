use crate::prehandle::auth::Guest;
use crate::sdk;
use crate::Context;
use crate::VERSION_INFO;
use sdk::system::get_system_info::GetSystemInfoReq;
use sdk::system::get_system_info::GetSystemInfoResp;
use std::sync::Arc;
use tihu_native::ErrNo;

pub async fn get_system_info(
    context: Arc<Context>,
    _guest: Guest,
    _get_system_info_req: GetSystemInfoReq,
) -> Result<GetSystemInfoResp, ErrNo> {
    let get_rsa_pub_key_content = context.get_rsa_pub_key_content().await.map_err(|err| {
        log::error!("获取公钥失败: {}", err);
        err
    })?;
    return Ok(GetSystemInfoResp {
        version: VERSION_INFO.to_string(),
        rsa_pub_key: get_rsa_pub_key_content.to_string(),
        turnstile_site_key: context
            .config
            .turnstile
            .as_ref()
            .map(|turnstile| turnstile.site_key.clone()),
    });
}
