use serde::{Deserialize, Serialize};
use tihu::Api;
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
}
