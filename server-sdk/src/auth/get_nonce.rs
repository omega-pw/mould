use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::SharedString;

pub const GET_NONCE_API: &str = "/api/auth/getNonce";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetNonceReq {}

pub type GetNonceResp = String;

pub struct GetNonceApi;
impl Api for GetNonceApi {
    type Input = GetNonceReq;
    type Output = GetNonceResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(GET_NONCE_API);
    }
}
