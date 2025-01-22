use crate::get_context;
use crate::model::external_user::enums::ProviderType;
use crate::model::user::enums::UserSource;
use crate::model::user::UserOpt;
use crate::sdk;
use crate::service::base::ExternalUserBaseService;
use crate::service::base::SystemUserBaseService;
use crate::service::base::UserBaseService;
use sdk::user::read_user::ExternalUser;
use sdk::user::read_user::ReadUserReq;
use sdk::user::read_user::ReadUserResp;
use sdk::user::read_user::SystemUser;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

fn to_sdk_provider_type(val: ProviderType) -> sdk::user::enums::ProviderType {
    match val {
        ProviderType::Openid => sdk::user::enums::ProviderType::Openid,
        ProviderType::Oauth2 => sdk::user::enums::ProviderType::Oauth2,
    }
}

pub async fn read_user(
    org_id: Id,
    _user: crate::middleware::auth::User,
    read_user_req: ReadUserReq,
) -> Result<ReadUserResp, ErrNo> {
    let ReadUserReq { id } = read_user_req;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let user_base_service = UserBaseService::new(&transaction);
    let params = UserOpt {
        org_id: Some(org_id),
        id: Some(id),
        ..UserOpt::empty()
    };
    let user_opt = user_base_service.query_user_one(&params).await?;
    if let Some(user) = user_opt {
        let user_source = match user.user_source {
            UserSource::System => {
                let system_user_base_service = SystemUserBaseService::new(&transaction);
                let system_user_opt = system_user_base_service.read_system_user(user.id).await?;
                let system_user = system_user_opt.ok_or_else(|| -> ErrNo {
                    ErrNo::CommonError(LightString::from_static("不存在此用户！"))
                })?;
                sdk::user::read_user::UserSource::System(SystemUser {
                    id: system_user.id,
                    email: system_user.email,
                    created_time: system_user.created_time,
                    last_modified_time: system_user.last_modified_time,
                })
            }
            UserSource::External => {
                let external_user_base_service = ExternalUserBaseService::new(&transaction);
                let external_user_opt = external_user_base_service
                    .read_external_user(user.id)
                    .await?;
                let external_user = external_user_opt.ok_or_else(|| -> ErrNo {
                    ErrNo::CommonError(LightString::from_static("不存在此用户！"))
                })?;
                sdk::user::read_user::UserSource::External(ExternalUser {
                    id: external_user.id,
                    provider_type: to_sdk_provider_type(external_user.provider_type),
                    provider: external_user.provider,
                    openid: external_user.openid,
                    detail: external_user.detail,
                    created_time: external_user.created_time,
                    last_modified_time: external_user.last_modified_time,
                })
            }
        };
        return Ok(Some(sdk::user::read_user::User {
            id: user.id.into(),
            user_source: user_source,
            name: user.name.into(),
            avatar_url: user.avatar_url.into(),
            created_time: user.created_time.into(),
            last_modified_time: user.last_modified_time.into(),
        }));
    } else {
        return Ok(None);
    }
}
