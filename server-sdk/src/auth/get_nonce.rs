use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;

pub const GET_NONCE_API: &str = "/api/auth/getNonce";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetNonceReq {}

pub type GetNonceResp = String;

pub struct GetNonceApi;
impl Api for GetNonceApi {
    type Input = GetNonceReq;
    type Output = GetNonceResp;
    fn namespace() -> LightString {
        return LightString::from_static(GET_NONCE_API);
    }
}
