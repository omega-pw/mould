use super::cache_session_info;
use super::check_nonce;
use super::decrypt_base64_data_by_rsa_pri_key;
use crate::get_context;
use crate::middleware::auth::AuthMethod;
use crate::middleware::auth::Guest;
use crate::middleware::auth::SessionInfo;
use crate::model::organization::Organization;
use crate::model::system_user::SystemUser;
use crate::model::system_user::SystemUserOpt;
use crate::model::user::enums::UserSource;
use crate::model::user::User;
use crate::model::user::UserOpt;
use crate::native_common;
use crate::sdk;
use crate::service::base::OrganizationBaseService;
use crate::service::base::SystemUserBaseService;
use crate::service::base::UserBaseService;
use chrono::Utc;
use native_common::cache::AsyncCache;
use native_common::utils::decrypt_by_base64;
use native_common::utils::encrypt_by_base64;
use native_common::utils::sha512;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::User as SdkUser;
use sdk::auth::register::RegisterApi;
use sdk::auth::register::RegisterReq;
use sdk::auth::register::RegisterResp;
use tihu::validator::ValidateEmail;
use tihu::Api;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn register(guest: Guest, register_req: RegisterReq) -> Result<RegisterResp, ErrNo> {
    if let Err(err) = RegisterApi::validate_input(&register_req) {
        return Err(ErrNo::CommonError(err));
    }
    let nonce_ok = check_nonce(&register_req.nonce).await?;
    if !nonce_ok {
        return Err(ErrNo::NotAllowed);
    }
    if register_req.captcha.is_empty() {
        return Err(ErrNo::CommonError(LightString::from_static(
            "验证码不能为空!",
        )));
    }
    let mut captcha = register_req.captcha.into_bytes();
    for item in &mut captcha {
        item.make_ascii_lowercase();
    }
    let context = get_context()?;
    let cache_mgr = context.get_cache_mgr().await?;
    let mut captcha2 = cache_mgr
        .get(&(String::from("register-captcha-") + &guest.session_id.to_string()).into_bytes())
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
            &register_req.account,
            "邮箱",
            register_req.nonce.as_bytes(),
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
            &register_req.auth_key,
            "密码",
            register_req.nonce.as_bytes(),
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
    let user_base_service = UserBaseService::new(&transaction);
    let system_user_base_service = SystemUserBaseService::new(&transaction);
    let count = system_user_base_service
        .query_system_user_count(&SystemUserOpt {
            email: Some(email.clone()),
            ..SystemUserOpt::empty()
        })
        .await?;
    if 0 < count {
        return Err(ErrNo::CommonError(LightString::from_static("该邮箱已注册")));
    }
    let user_opt = user_base_service.query_user_one(&UserOpt::empty()).await?;
    let curr_time = Utc::now();
    let org_id = if user_opt.is_none() {
        //第一个用户
        let organization_base_service = OrganizationBaseService::new(&transaction);
        let org_id = context.new_id();
        organization_base_service
            .insert_organization(&Organization {
                id: org_id,
                name: String::from("默认组织"),
                created_time: curr_time,
                last_modified_time: curr_time,
            })
            .await?;
        Some(org_id)
    } else {
        None
    };
    let user_id = context.new_id();
    let curr_time = Utc::now();
    let user = User {
        id: user_id,
        org_id: org_id,                  //组织id
        user_source: UserSource::System, //用户来源
        name: email.clone(),
        avatar_url: None,
        created_time: curr_time,
        last_modified_time: curr_time,
    };
    let system_user = SystemUser {
        id: user_id,
        email: email.clone(),
        user_random_value: register_req.user_random_value,
        hashed_auth_key: hashed_auth_key,
        created_time: curr_time,
        last_modified_time: curr_time,
    };
    user_base_service.insert_user(&user).await?;
    system_user_base_service
        .insert_system_user(&system_user)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    let session_info = SessionInfo {
        auth_method: AuthMethod::System,
        user_id: user_id,
        org_id: org_id,
    };
    cache_session_info(guest.session_id, &session_info).await?;
    return Ok(Some(SdkUser {
        id: user_id,
        org_id: org_id,
        name: Some(user.name),
        avatar_url: user.avatar_url,
        auth_source: AuthSource::System {
            email: system_user.email,
            user_random_value: system_user.user_random_value,
        },
    }));
}
