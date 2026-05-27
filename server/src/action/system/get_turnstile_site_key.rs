use crate::get_context;
use crate::middleware::auth::Guest;
use crate::sdk;
use sdk::system::get_turnstile_site_key::GetTurnstileSiteKeyReq;
use sdk::system::get_turnstile_site_key::GetTurnstileSiteKeyResp;
use tihu_native::ErrNo;

pub async fn get_turnstile_site_key(
    _guest: Guest,
    _get_turnstile_site_key_req: GetTurnstileSiteKeyReq,
) -> Result<GetTurnstileSiteKeyResp, ErrNo> {
    let context = get_context()?;
    return Ok(GetTurnstileSiteKeyResp {
        site_key: context
            .config
            .turnstile
            .as_ref()
            .map(|turnstile| turnstile.site_key.clone()),
    });
}
