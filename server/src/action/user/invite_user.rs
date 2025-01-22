use crate::get_context;
use crate::middleware::auth::User;
use crate::model::user::UserProperty;
use crate::sdk;
use crate::service::base::UserBaseService;
use chrono::Utc;
use sdk::user::invite_user::InviteUserReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn invite_user(
    org_id: Id,
    _user: User,
    invite_user_req: InviteUserReq,
) -> Result<(), ErrNo> {
    let InviteUserReq { user_id } = invite_user_req;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let user_base_service = UserBaseService::new(&transaction);
    let user_opt = user_base_service.read_user(user_id).await?;
    let user = user_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("不存在此用户！"))
    })?;
    if let Some(existed_org_id) = user.org_id {
        if existed_org_id == org_id {
            return Ok(());
        } else {
            return Err(ErrNo::CommonError(LightString::from_static(
                "该用户已有归属组织！",
            )));
        }
    }
    let curr_time = Utc::now();
    let changes = vec![
        UserProperty::OrgId(Some(org_id)),
        UserProperty::LastModifiedTime(curr_time),
    ];
    user_base_service.update_user(user_id, &changes).await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(());
}
