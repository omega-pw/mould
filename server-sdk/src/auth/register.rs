use super::get_curr_user::GetCurrUserResp;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;
use validator::Validate;

pub const REGISTER_API: &str = "/api/auth/register";

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct RegisterReq {
    pub nonce: String,
    pub account: String,
    pub user_random_value: String, //随机数
    pub auth_key: String,          //授权秘钥
    pub captcha: String,
}

pub type RegisterResp = GetCurrUserResp;

pub struct RegisterApi;
impl Api for RegisterApi {
    type Input = RegisterReq;
    type Output = RegisterResp;
    fn namespace() -> LightString {
        return LightString::from_static(REGISTER_API);
    }
}
