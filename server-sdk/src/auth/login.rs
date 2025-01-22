use super::get_curr_user::GetCurrUserResp;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;
use validator::Validate;

pub const LOGIN_API: &str = "/api/auth/login";

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct LoginReq {
    pub nonce: String,
    pub account: String,
    pub auth_key: String,
}

pub type LoginResp = GetCurrUserResp;

pub struct LoginApi;
impl Api for LoginApi {
    type Input = LoginReq;
    type Output = LoginResp;
    fn namespace() -> LightString {
        return LightString::from_static(LOGIN_API);
    }
}
