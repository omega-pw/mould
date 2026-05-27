use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::api::CacheMeta;
use tihu::SharedString;

pub const GET_RSA_PUB_KEY_API: &str = "/api/auth/getRsaPubKey";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRsaPubKeyReq {}

pub type GetRsaPubKeyResp = String;

pub struct GetRsaPubKeyApi;
impl Api for GetRsaPubKeyApi {
    type Input = GetRsaPubKeyReq;
    type Output = GetRsaPubKeyResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(GET_RSA_PUB_KEY_API);
    }
    fn retryable() -> bool {
        return true;
    }
    fn get_cache_meta(_: &Self::Input) -> Option<CacheMeta> {
        return Some(CacheMeta {
            key: SharedString::from_static(GET_RSA_PUB_KEY_API),
            ttl: Some(5 * 60 * 1000),
        });
    }
}
