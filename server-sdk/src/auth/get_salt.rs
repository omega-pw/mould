use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;
use validator::Validate;

pub const GET_SALT_API: &str = "/api/auth/getSalt";

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct GetSaltReq {
    #[validate(email)]
    pub account: String,
}

pub type GetSaltResp = String;

pub struct GetSaltApi;
impl Api for GetSaltApi {
    type Input = GetSaltReq;
    type Output = GetSaltResp;
    fn namespace() -> LightString {
        return LightString::from_static(GET_SALT_API);
    }
    fn validate_input(req: &Self::Input) -> Result<(), LightString> {
        return req.validate().map_err(|err| -> LightString {
            log::error!("邮箱格式不正确: {:?}", err);
            return LightString::from_static("邮箱格式不正确");
        });
    }
}
