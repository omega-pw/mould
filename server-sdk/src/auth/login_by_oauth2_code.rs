use super::get_curr_user::GetCurrUserResp;
use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;

pub const LOGIN_BY_OAUTH2_CODE_API: &str = "/api/auth/loginByOauth2Code";

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginByOauth2CodeReq {
    pub provider: String,
    pub code: String,
    pub pkce_verifier: Option<String>,
}

pub type LoginByOauth2CodeResp = GetCurrUserResp;

pub struct LoginByOauth2CodeApi;
impl Api for LoginByOauth2CodeApi {
    type Input = LoginByOauth2CodeReq;
    type Output = LoginByOauth2CodeResp;
    fn namespace() -> LightString {
        return LightString::from_static(LOGIN_BY_OAUTH2_CODE_API);
    }
}
