use crate::get_context;
use crate::log;
use crate::middleware::auth::Guest;
use crate::model::captcha::enums::ReceiverType;
use crate::model::captcha::enums::Scene;
use crate::model::captcha::Captcha;
use crate::model::captcha::CaptchaOpt;
use crate::model::captcha::CaptchaProperty;
use crate::native_common;
use crate::sdk;
use crate::service::base::CaptchaBaseService;
use chrono::Utc;
use native_common::cache::AsyncCache;
use native_common::cache::EliminateType;
use native_common::utils::fill_random;
use native_common::utils::send_mail;
use native_common::utils::DEFAULT_SEED;
use sdk::auth::send_email_captcha::Scene as SdkScene;
use sdk::auth::send_email_captcha::SendEmailCaptchaReq;
use sdk::auth::send_email_captcha::SendEmailCaptchaResp;
use tera::Tera;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

fn from_sdk_scene(val: SdkScene) -> Scene {
    match val {
        SdkScene::Register => Scene::Register,
        SdkScene::ResetPassword => Scene::ResetPassword,
    }
}

pub async fn send_email_captcha(
    guest: Guest,
    send_email_captcha_req: SendEmailCaptchaReq,
) -> Result<SendEmailCaptchaResp, ErrNo> {
    let mut captcha = vec!['0'; 8];
    fill_random(&mut captcha, DEFAULT_SEED);
    let captcha: String = captcha.into_iter().collect();
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let captcha_base_service = CaptchaBaseService::new(&transaction);
    let existed = captcha_base_service
        .query_captcha_one(&CaptchaOpt {
            receiver_type: Some(ReceiverType::Email),
            receiver: Some(send_email_captcha_req.email.clone()),
            scene: Some(from_sdk_scene(send_email_captcha_req.scene)),
            ..CaptchaOpt::empty()
        })
        .await?;
    if let Some(existed) = existed {
        let curr_time = Utc::now();
        let changes: Vec<CaptchaProperty> = vec![
            CaptchaProperty::Captcha(captcha.clone()),
            CaptchaProperty::LastModifiedTime(curr_time),
        ];
        captcha_base_service
            .update_captcha(existed.id, &changes)
            .await?;
    } else {
        let curr_time = Utc::now();
        let id = context.new_id();
        let captcha = Captcha {
            id: id,
            receiver_type: ReceiverType::Email,
            receiver: send_email_captcha_req.email.clone(),
            scene: from_sdk_scene(send_email_captcha_req.scene),
            captcha: captcha.clone(),
            created_time: curr_time,
            last_modified_time: curr_time,
        };
        captcha_base_service.insert_captcha(&captcha).await?;
    }
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    match send_email_captcha_req.scene {
        SdkScene::Register => {
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
        SdkScene::ResetPassword => {
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
