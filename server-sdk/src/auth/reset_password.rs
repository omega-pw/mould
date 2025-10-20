use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::SharedString;
use validator::Validate;

pub const RESET_PASSWORD_API: &str = "/api/auth/resetPassword";

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct ResetPasswordReq {
    pub nonce: String,
    pub account: String,
    pub auth_key: String,
    pub captcha: String,
}

pub type ResetPasswordResp = ();

pub struct ResetPasswordApi;
impl Api for ResetPasswordApi {
    type Input = ResetPasswordReq;
    type Output = ResetPasswordResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(RESET_PASSWORD_API);
    }
}
