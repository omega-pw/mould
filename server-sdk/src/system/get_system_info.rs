use serde::{Deserialize, Serialize};
use tihu::api::CacheMeta;
use tihu::Api;
use tihu::SharedString;

pub const GET_SYSTEM_INFO_API: &str = "/api/system/getSystemInfo";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSystemInfoReq {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetSystemInfoResp {
    pub version: String,
    pub rsa_pub_key: String,
    pub turnstile_site_key: Option<String>,
}

pub struct GetSystemInfoApi;
impl Api for GetSystemInfoApi {
    type Input = GetSystemInfoReq;
    type Output = GetSystemInfoResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(GET_SYSTEM_INFO_API);
    }
    fn retryable() -> bool {
        return true;
    }
    fn get_cache_meta(_: &Self::Input) -> Option<CacheMeta> {
        return Some(CacheMeta {
            key: SharedString::from_static(GET_SYSTEM_INFO_API),
            ttl: Some(5 * 60 * 1000),
        });
    }
}
