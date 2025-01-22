use super::check_nonce;
use super::decrypt_base64_data_by_rsa_pri_key;
use crate::get_context;
use crate::middleware::auth::Guest;
use crate::model::system_user::SystemUserOpt;
use crate::model::system_user::SystemUserProperty;
use crate::native_common;
use crate::sdk;
use crate::service::base::SystemUserBaseService;
use chrono::Utc;
use native_common::cache::AsyncCache;
use native_common::utils::decrypt_by_base64;
use native_common::utils::encrypt_by_base64;
use native_common::utils::sha512;
use sdk::auth::reset_password::ResetPasswordApi;
use sdk::auth::reset_password::ResetPasswordReq;
use sdk::auth::reset_password::ResetPasswordResp;
use tihu::validator::ValidateEmail;
use tihu::Api;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn reset_password(
    guest: Guest,
    reset_password_req: ResetPasswordReq,
) -> Result<ResetPasswordResp, ErrNo> {
    if let Err(err) = ResetPasswordApi::validate_input(&reset_password_req) {
        return Err(ErrNo::CommonError(err));
    }
    let nonce_ok = check_nonce(&reset_password_req.nonce).await?;
    if !nonce_ok {
        return Err(ErrNo::NotAllowed);
    }
    if reset_password_req.captcha.is_empty() {
        return Err(ErrNo::CommonError(LightString::from_static(
            "验证码不能为空!",
        )));
    }
    let mut captcha = reset_password_req.captcha.into_bytes();
    for item in &mut captcha {
        item.make_ascii_lowercase();
    }
    let context = get_context()?;
    let cache_mgr = context.get_cache_mgr().await?;
    let mut captcha2 = cache_mgr
        .get(
            &(String::from("reset-password-captcha-") + &guest.session_id.to_string()).into_bytes(),
        )
        .await?
        .ok_or_else(|| ErrNo::CommonError(LightString::from_static("验证码不存在或已过期!")))?;
    for item in &mut captcha2 {
        item.make_ascii_lowercase();
    }
    if captcha != captcha2 {
        return Err(ErrNo::CommonError(LightString::from_static(
            "验证码不正确!",
        )));
    }
    let rsa_pri_key = &context.get_rsa_pri_key().await?;
    let email = String::from_utf8_lossy(
        &decrypt_base64_data_by_rsa_pri_key(
            &reset_password_req.account,
            "邮箱",
            reset_password_req.nonce.as_bytes(),
            rsa_pri_key,
        )
        .map_err(ErrNo::CommonError)?,
    )
    .into_owned();
    if !ValidateEmail::validate_email(&email) {
        return Err(ErrNo::CommonError("邮箱格式不正确".into()));
    }
    let auth_key = String::from_utf8_lossy(
        &decrypt_base64_data_by_rsa_pri_key(
            &reset_password_req.auth_key,
            "密码",
            reset_password_req.nonce.as_bytes(),
            rsa_pri_key,
        )
        .map_err(ErrNo::CommonError)?,
    )
    .into_owned();
    let auth_key = decrypt_by_base64(&auth_key).map_err(|err| {
        log::error!("解码授权key失败: {:?}", err);
        return ErrNo::CommonError(LightString::from_static("解码授权key失败！"));
    })?;
    let hashed_auth_key = encrypt_by_base64(&sha512(&auth_key)).map_err(ErrNo::CommonError)?;

    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let system_user_base_service = SystemUserBaseService::new(&transaction);
    let system_user_opt = system_user_base_service
        .query_system_user_one(&SystemUserOpt {
            email: Some(email.clone()),
            ..SystemUserOpt::empty()
        })
        .await?;
    let system_user =
        system_user_opt.ok_or_else(|| ErrNo::CommonError(LightString::from("用户不存在！")))?;
    let user_id = system_user.id;
    let curr_time = Utc::now();
    let changes: Vec<SystemUserProperty> = vec![
        SystemUserProperty::HashedAuthKey(hashed_auth_key),
        SystemUserProperty::LastModifiedTime(curr_time),
    ];
    system_user_base_service
        .update_system_user(user_id, &changes)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(());
}
