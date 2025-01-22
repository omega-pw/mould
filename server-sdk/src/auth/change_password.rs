use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;

pub const CHANGE_PASSWORD_API: &str = "/api/auth/changePassword";

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePasswordReq {
    pub nonce: String,
    pub old_auth_key: String,
    pub new_auth_key: String,
}

pub type ChangePasswordResp = ();

pub struct ChangePasswordApi;
impl Api for ChangePasswordApi {
    type Input = ChangePasswordReq;
    type Output = ChangePasswordResp;
    fn namespace() -> LightString {
        return LightString::from_static(CHANGE_PASSWORD_API);
    }
}
