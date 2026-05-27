use serde::{Deserialize, Serialize};
use tihu::api::CacheMeta;
use tihu::Api;
use tihu::SharedString;

pub const GET_TURNSTILE_SITE_KEY_API: &str = "/api/system/getTurnstileSiteKey";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTurnstileSiteKeyReq {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetTurnstileSiteKeyResp {
    pub site_key: Option<String>,
}

pub struct GetTurnstileSiteKeyApi;
impl Api for GetTurnstileSiteKeyApi {
    type Input = GetTurnstileSiteKeyReq;
    type Output = GetTurnstileSiteKeyResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(GET_TURNSTILE_SITE_KEY_API);
    }
    fn retryable() -> bool {
        return true;
    }
    fn get_cache_meta(_: &Self::Input) -> Option<CacheMeta> {
        return Some(CacheMeta {
            key: SharedString::from_static(GET_TURNSTILE_SITE_KEY_API),
            ttl: Some(5 * 60 * 1000),
        });
    }
}
