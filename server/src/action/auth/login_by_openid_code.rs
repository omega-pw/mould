use super::cache_session_info;
use crate::get_context;
use crate::middleware::auth::AuthMethod;
use crate::middleware::auth::Guest;
use crate::middleware::auth::OpenidToken;
use crate::middleware::auth::SessionInfo;
use crate::model::external_user::enums::ProviderType;
use crate::model::external_user::ExternalUser;
use crate::model::external_user::ExternalUserOpt;
use crate::model::external_user::ExternalUserProperty;
use crate::model::organization::Organization;
use crate::model::user::enums::UserSource;
use crate::model::user::User;
use crate::model::user::UserOpt;
use crate::model::user::UserProperty;
use crate::sdk;
use crate::service::base::ExternalUserBaseService;
use crate::service::base::OrganizationBaseService;
use crate::service::base::UserBaseService;
use chrono::Utc;
use openid::Token;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::User as SdkUser;
use sdk::auth::login_by_openid_code::LoginByOpenidCodeReq;
use sdk::auth::login_by_openid_code::LoginByOpenidCodeResp;
use serde::{Deserialize, Serialize};
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

#[derive(Serialize, Deserialize, Debug)]
struct WechatTokenResp {
    access_token: String,
    expires_in: f64,
    refresh_token: String,
    openid: String,
    scope: String,
    unionid: String,
}

pub async fn login_by_openid_code(
    guest: Guest,
    login_by_openid_code_req: LoginByOpenidCodeReq,
) -> Result<LoginByOpenidCodeResp, ErrNo> {
    let LoginByOpenidCodeReq { provider, code } = login_by_openid_code_req;
    let context = get_context()?;
    let (openid_client, _) = context.get_openid_client(&provider)?;
    let mut token: Token = openid_client
        .request_token(&code)
        .await
        .map_err(|err| {
            log::error!("Get token by code failed, {:?}", err);
            ErrNo::ApiError(err.into())
        })?
        .into();
    let id_token = token
        .id_token
        .as_mut()
        .ok_or_else(|| ErrNo::LoginRequired)?;
    //必须进行decode_token操作，否则给request_userinfo操作用报错
    openid_client.decode_token(id_token).map_err(|err| {
        log::error!("Decode token failed, {:?}", err);
        ErrNo::ApiError(err.into())
    })?;
    // openid_client.validate_token(id_token, None, None)?;
    let userinfo = openid_client
        .request_userinfo(&token)
        .await
        .map_err(|err| {
            log::error!("Get userinfo by token failed, {:?}", err);
            ErrNo::ApiError(err.into())
        })?;
    let openid = userinfo.sub.clone().ok_or_else(|| {
        ErrNo::CommonError(LightString::from(
            "No property \"sub\" found in user infomation!",
        ))
    })?;
    let user_detail = serde_json::to_string(&userinfo).map_err(ErrNo::SerializeError)?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let user_base_service = UserBaseService::new(&transaction);
    let external_user_base_service = ExternalUserBaseService::new(&transaction);
    let external_user_opt = external_user_base_service
        .query_external_user_one(&ExternalUserOpt {
            provider_type: Some(ProviderType::Openid),
            provider: Some(provider.clone()),
            openid: Some(openid.clone()),
            ..ExternalUserOpt::empty()
        })
        .await?;
    let user_pair_opt = if let Some(external_user) = external_user_opt {
        let user_opt = user_base_service.read_user(external_user.id).await?;
        if let Some(user) = user_opt {
            Some((user, external_user))
        } else {
            None
        }
    } else {
        None
    };
    let name = userinfo
        .nickname
        .or(userinfo.preferred_username)
        .or(userinfo.name)
        .or(userinfo.given_name)
        .or(userinfo.family_name)
        .or(userinfo.middle_name)
        .unwrap_or_else(|| String::from("匿名用户"));
    let avatar_url = userinfo.picture.map(|picture| picture.to_string());
    let curr_time = Utc::now();
    let (session_info, curr_user) = if let Some((user, external_user)) = user_pair_opt {
        let user_id = user.id;
        let org_id = user.org_id;
        let changes: Vec<UserProperty> = vec![
            UserProperty::Name(name),
            UserProperty::AvatarUrl(avatar_url),
            UserProperty::LastModifiedTime(curr_time),
        ];
        user_base_service.update_user(user_id, &changes).await?;
        let changes: Vec<ExternalUserProperty> = vec![
            ExternalUserProperty::Detail(Some(user_detail.clone())),
            ExternalUserProperty::LastModifiedTime(curr_time),
        ];
        external_user_base_service
            .update_external_user(user_id, &changes)
            .await?;
        (
            SessionInfo {
                auth_method: AuthMethod::Openid(OpenidToken {
                    provider: provider.clone(),
                    bearer: token.bearer.clone(),
                    openid: external_user.openid,
                }),
                user_id: user_id,
                org_id: org_id,
            },
            SdkUser {
                id: user_id,
                org_id: org_id,
                name: Some(user.name),
                avatar_url: user.avatar_url,
                auth_source: AuthSource::External {
                    provider_type: sdk::user::enums::ProviderType::Openid,
                    provider: provider,
                    openid: openid,
                    detail: user_detail,
                },
            },
        )
    } else {
        let user_opt = user_base_service.query_user_one(&UserOpt::empty()).await?;
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
        let user = User {
            id: user_id,
            org_id: org_id,
            user_source: UserSource::External,
            name: name,
            avatar_url: avatar_url,
            created_time: curr_time,
            last_modified_time: curr_time,
        };
        let external_user = ExternalUser {
            id: user_id,
            provider_type: ProviderType::Openid,
            provider: provider.clone(),
            openid: openid.clone(),
            detail: Some(user_detail.clone()),
            created_time: curr_time,
            last_modified_time: curr_time,
        };
        user_base_service.insert_user(&user).await?;
        external_user_base_service
            .insert_external_user(&external_user)
            .await?;
        (
            SessionInfo {
                auth_method: AuthMethod::Openid(OpenidToken {
                    provider: provider.clone(),
                    bearer: token.bearer.clone(),
                    openid: openid.clone(),
                }),
                user_id: user_id,
                org_id: org_id,
            },
            SdkUser {
                id: user_id,
                org_id: org_id,
                name: Some(user.name),
                avatar_url: user.avatar_url,
                auth_source: AuthSource::External {
                    provider_type: sdk::user::enums::ProviderType::Openid,
                    provider: provider,
                    openid: openid,
                    detail: user_detail,
                },
            },
        )
    };
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    cache_session_info(guest.session_id, &session_info).await?;
    return Ok(Some(curr_user));
}
