use super::get_curr_user::GetCurrUserResp;
use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;

pub const LOGIN_BY_OPENID_CODE_API: &str = "/api/auth/loginByOpenidCode";

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginByOpenidCodeReq {
    pub provider: String,
    pub code: String,
}

pub type LoginByOpenidCodeResp = GetCurrUserResp;

pub struct LoginByOpenidCodeApi;
impl Api for LoginByOpenidCodeApi {
    type Input = LoginByOpenidCodeReq;
    type Output = LoginByOpenidCodeResp;
    fn namespace() -> LightString {
        return LightString::from_static(LOGIN_BY_OPENID_CODE_API);
    }
}
