use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::SharedString;
use validator::Validate;

pub const SEND_EMAIL_CAPTCHA_API: &str = "/api/auth/sendEmailCaptcha";

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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
    fn namespace() -> SharedString {
        return SharedString::from_static(SEND_EMAIL_CAPTCHA_API);
    }
    fn validate_input(req: &Self::Input) -> Result<(), SharedString> {
        return req.validate().map_err(|err| -> SharedString {
            log::error!("邮箱格式不正确: {:?}", err);
            return SharedString::from_static("邮箱格式不正确");
        });
    }
}
