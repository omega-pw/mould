use super::check_nonce;
use super::decrypt_base64_data_by_rsa_pri_key;
use crate::get_context;
use crate::middleware::auth::User;
use crate::model::system_user::SystemUser;
use crate::model::system_user::SystemUserProperty;
use crate::native_common;
use crate::sdk;
use crate::service::base::SystemUserBaseService;
use chrono::Utc;
use native_common::utils::decrypt_by_base64;
use native_common::utils::encrypt_by_base64;
use native_common::utils::sha512;
use sdk::auth::change_password::ChangePasswordReq;
use sdk::auth::change_password::ChangePasswordResp;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn change_password(
    _org_id: Id,
    user: User,
    change_password_req: ChangePasswordReq,
) -> Result<ChangePasswordResp, ErrNo> {
    let ChangePasswordReq {
        nonce,
        old_auth_key,
        new_auth_key,
    } = change_password_req;
    let nonce_ok = check_nonce(&nonce).await?;
    if !nonce_ok {
        return Err(ErrNo::NotAllowed);
    }
    let context = get_context()?;
    let rsa_pri_key = &context.get_rsa_pri_key().await?;
    let old_auth_key = String::from_utf8_lossy(
        &decrypt_base64_data_by_rsa_pri_key(
            &old_auth_key,
            "旧密码",
            nonce.as_bytes(),
            &rsa_pri_key,
        )
        .map_err(ErrNo::CommonError)?,
    )
    .into_owned();
    let new_auth_key = String::from_utf8_lossy(
        &decrypt_base64_data_by_rsa_pri_key(
            &new_auth_key,
            "新密码",
            nonce.as_bytes(),
            &rsa_pri_key,
        )
        .map_err(ErrNo::CommonError)?,
    )
    .into_owned();
    let old_auth_key = decrypt_by_base64(&old_auth_key).map_err(|err| {
        log::error!("解码授权key失败: {:?}", err);
        return ErrNo::CommonError(LightString::from_static("解码授权key失败！"));
    })?;
    let old_hashed_auth_key =
        encrypt_by_base64(&sha512(&old_auth_key)).map_err(ErrNo::CommonError)?;
    let new_auth_key = decrypt_by_base64(&new_auth_key).map_err(|err| {
        log::error!("解码授权key失败: {:?}", err);
        return ErrNo::CommonError(LightString::from_static("解码授权key失败！"));
    })?;
    let new_hashed_auth_key =
        encrypt_by_base64(&sha512(&new_auth_key)).map_err(ErrNo::CommonError)?;

    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let system_user_base_service = SystemUserBaseService::new(&transaction);
    let system_user_opt = system_user_base_service
        .read_system_user(user.user_id)
        .await?;
    let system_user = system_user_opt
        .ok_or_else(|| ErrNo::CommonError(LightString::from_static("数据异常，当前用户不存在")))?;
    let SystemUser {
        id,
        hashed_auth_key,
        ..
    } = system_user;
    if old_hashed_auth_key != hashed_auth_key {
        return Err(ErrNo::CommonError(LightString::from_static("旧密码不正确")));
    }
    let curr_time = Utc::now();
    system_user_base_service
        .update_system_user(
            id,
            &vec![
                SystemUserProperty::HashedAuthKey(new_hashed_auth_key),
                SystemUserProperty::LastModifiedTime(curr_time),
            ],
        )
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(());
}
