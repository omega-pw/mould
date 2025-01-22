use crate::get_context;
use crate::log;
use crate::middleware::auth::Guest;
use crate::native_common;
use crate::sdk;
use native_common::cache::AsyncCache;
use native_common::cache::EliminateType;
use native_common::utils::send_mail;
use sdk::auth::send_email_captcha::Scene;
use sdk::auth::send_email_captcha::SendEmailCaptchaReq;
use sdk::auth::send_email_captcha::SendEmailCaptchaResp;
use tera::Tera;
use tihu::base62;
use tihu::LightString;
use tihu_native::ErrNo;
use uuid::Uuid;

pub async fn send_email_captcha(
    guest: Guest,
    send_email_captcha_req: SendEmailCaptchaReq,
) -> Result<SendEmailCaptchaResp, ErrNo> {
    match send_email_captcha_req.scene {
        Scene::Register => {
            let captcha = base62::encode(&Uuid::new_v4().as_u128().to_be_bytes());
            let context = get_context()?;
            let mut data = tera::Context::new();
            data.insert("captcha", &captcha);
            let mail_content = Tera::default()
                .render_str(&context.config.email_template.register_captcha, &data)
                .map_err(|err| {
                    log::error!("组装注册验证码邮件内容失败: {}", err);
                    return ErrNo::CommonError(LightString::from_static(
                        "Failed to assemble registration verification code email content",
                    ));
                })?;
            let cache_mgr = context.get_cache_mgr().await?;
            cache_mgr
                .set(
                    &(String::from("register-captcha-") + &guest.session_id.to_string())
                        .into_bytes(),
                    &captcha.into_bytes(),
                    EliminateType::Expire(5 * 60 * 1000), //5分钟过期
                )
                .await
                .map_err(|err| {
                    log::error!("存储注册验证码失败: {}", err);
                    return err;
                })?;
            let email_account = context.config.email_account.clone();
            return send_mail(
                &email_account.mail_host,
                email_account.mail_port,
                email_account.username.clone(),
                email_account.password.clone(),
                Some(email_account.name.clone()),
                &email_account.address,
                None,
                &send_email_captcha_req.email,
                "欢迎注册",
                mail_content,
            )
            .await
            .map_err(|err| {
                log::error!("通过邮件发送注册验证码失败: {}", err);
                return ErrNo::CommonError(LightString::from_static("发送注册验证码失败"));
            });
        }
        Scene::ResetPassword => {
            let captcha = base62::encode(&Uuid::new_v4().as_u128().to_be_bytes());
            let context = get_context()?;
            let mut data = tera::Context::new();
            data.insert("captcha", &captcha);
            let mail_content = Tera::default()
                .render_str(&context.config.email_template.reset_password_captcha, &data)
                .map_err(|err| {
                    log::error!("组装重置密码验证码邮件内容失败: {}", err);
                    return ErrNo::CommonError(LightString::from_static(
                        "Failed to assemble reset verification code email content",
                    ));
                })?;
            let cache_mgr = context.get_cache_mgr().await?;
            cache_mgr
                .set(
                    &(String::from("reset-password-captcha-") + &guest.session_id.to_string())
                        .into_bytes(),
                    &captcha.into_bytes(),
                    EliminateType::Expire(5 * 60 * 1000), //5分钟过期
                )
                .await
                .map_err(|err| {
                    log::error!("存储重置密码验证码失败: {}", err);
                    return err;
                })?;
            let email_account = context.config.email_account.clone();
            return send_mail(
                &email_account.mail_host,
                email_account.mail_port,
                email_account.username.clone(),
                email_account.password.clone(),
                Some(email_account.name.clone()),
                &email_account.address,
                None,
                &send_email_captcha_req.email,
                "重置密码",
                mail_content,
            )
            .await
            .map_err(|err| {
                log::error!("通过邮件发送重置密码验证码失败: {}", err);
                return ErrNo::CommonError(LightString::from_static("发送重置密码验证码失败"));
            });
        }
    }
}
