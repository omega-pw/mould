use crate::get_context;
use crate::sdk;
use crate::VERSION_INFO;
use chrono::Utc;
use sdk::system::get_system_info::GetSystemInfoResp;
use tihu_native::ErrNo;

pub async fn get_system_info() -> Result<GetSystemInfoResp, ErrNo> {
    let context = get_context()?;
    return Ok(GetSystemInfoResp {
        version: VERSION_INFO.to_string(),
        current_time: Utc::now(),
        turnstile_site_key: context
            .config
            .turnstile
            .as_ref()
            .map(|turnstile| turnstile.site_key.clone()),
    });
}
