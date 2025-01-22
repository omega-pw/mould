use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;
use validator::Validate;

pub const SEND_EMAIL_CAPTCHA_API: &str = "/api/auth/sendEmailCaptcha";

#[derive(Serialize, Deserialize, Debug)]
pub enum Scene {
    Register,
    ResetPassword,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct SendEmailCaptchaReq {
    pub scene: Scene,
    #[validate(email)]
    pub email: String,
}

pub type SendEmailCaptchaResp = ();

pub struct SendEmailCaptchaApi;
impl Api for SendEmailCaptchaApi {
    type Input = SendEmailCaptchaReq;
    type Output = SendEmailCaptchaResp;
    fn namespace() -> LightString {
        return LightString::from_static(SEND_EMAIL_CAPTCHA_API);
    }
    fn validate_input(req: &Self::Input) -> Result<(), LightString> {
        return req.validate().map_err(|err| -> LightString {
            log::error!("邮箱格式不正确: {:?}", err);
            return LightString::from_static("邮箱格式不正确");
        });
    }
}
