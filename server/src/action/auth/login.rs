use super::cache_session_info;
use super::check_nonce;
use super::decrypt_base64_data_by_rsa_pri_key;
use crate::get_context;
use crate::middleware::auth::AuthMethod;
use crate::middleware::auth::Guest;
use crate::middleware::auth::SessionInfo;
use crate::model::system_user::SystemUserOpt;
use crate::native_common;
use crate::sdk;
use crate::service::base::SystemUserBaseService;
use crate::service::base::UserBaseService;
use native_common::utils::decrypt_by_base64;
use native_common::utils::encrypt_by_base64;
use native_common::utils::sha512;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::User as SdkUser;
use sdk::auth::login::LoginReq;
use sdk::auth::login::LoginResp;
use tihu::validator::ValidateEmail;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn login(guest: Guest, login_req: LoginReq) -> Result<LoginResp, ErrNo> {
    let nonce_ok = check_nonce(&login_req.nonce).await?;
    if !nonce_ok {
        return Err(ErrNo::TokenInvalid);
    }
    let context = get_context()?;
    let rsa_pri_key = &context.get_rsa_pri_key().await?;
    let email = String::from_utf8_lossy(
        &decrypt_base64_data_by_rsa_pri_key(
            &login_req.account,
            "邮箱",
            login_req.nonce.as_bytes(),
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
            &login_req.auth_key,
            "密码",
            login_req.nonce.as_bytes(),
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
    let mut list = system_user_base_service
        .query_system_user(
            1,
            2,
            &SystemUserOpt {
                email: Some(email.clone()),
                hashed_auth_key: Some(hashed_auth_key),
                ..SystemUserOpt::empty()
            },
        )
        .await?;
    if let Some(system_user) = list.pop() {
        if !list.is_empty() {
            log::warn!("数据异常：根据一个账号查到多条用户数据，邮箱: {:?}", email);
            return Err(ErrNo::CommonError(LightString::from_static(
                "用户名或密码错误！",
            )));
        }
        let user_id = system_user.id;
        let user_base_service = UserBaseService::new(&transaction);
        let user_opt = user_base_service.read_user(user_id).await?;
        let user = user_opt.ok_or_else(|| -> ErrNo {
            ErrNo::CommonError(LightString::from_static("不存在此用户！"))
        })?;
        let session_info = SessionInfo {
            auth_method: AuthMethod::System,
            user_id: user_id,
            org_id: user.org_id,
        };
        cache_session_info(guest.session_id, &session_info).await?;
        return Ok(Some(SdkUser {
            id: user_id,
            org_id: user.org_id,
            name: Some(user.name),
            avatar_url: user.avatar_url,
            auth_source: AuthSource::System {
                email: system_user.email,
                user_random_value: system_user.user_random_value,
            },
        }));
    } else {
        return Err(ErrNo::CommonError(LightString::from_static(
            "用户名或密码错误！",
        )));
    }
}
