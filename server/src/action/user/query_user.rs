use crate::get_context;
use crate::model::user::enums::UserSource;
use crate::model::user::User;
use crate::model::user::UserOpt;
use crate::sdk;
use crate::service::base::UserBaseService;
use sdk::user::query_user::QueryUserReq;
use tihu::pagination::PaginationList;
use tihu::Id;
use tihu::Pagination;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

fn from_sdk_user_source(val: sdk::user::enums::UserSource) -> UserSource {
    match val {
        sdk::user::enums::UserSource::System => UserSource::System,
        sdk::user::enums::UserSource::External => UserSource::External,
    }
}

fn to_sdk_user_source(val: UserSource) -> sdk::user::enums::UserSource {
    match val {
        UserSource::System => sdk::user::enums::UserSource::System,
        UserSource::External => sdk::user::enums::UserSource::External,
    }
}

pub async fn query_user(
    org_id: Id,
    _user: crate::middleware::auth::User,
    query_user_req: QueryUserReq,
) -> Result<PaginationList<sdk::user::query_user::User>, ErrNo> {
    let QueryUserReq {
        id,
        user_source,
        name,
        page_no,
        page_size,
    } = query_user_req;
    let params = UserOpt {
        org_id: Some(org_id),
        id: id.map(|v| v.into()),
        user_source: user_source.map(from_sdk_user_source),
        name: name.map(|v| v.into()),
        ..UserOpt::empty()
    };
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let user_base_service = UserBaseService::new(&transaction);
    let count = user_base_service.query_user_count(&params).await?;
    let pagination = Pagination::new(count, page_no.unwrap_or(1), page_size, None);
    let user_list = user_base_service
        .query_user(pagination.page_no, pagination.page_size, &params)
        .await?;
    let list = user_list
        .into_iter()
        .map(
            |User {
                 id,
                 user_source,
                 name,
                 avatar_url,
                 created_time,
                 last_modified_time,
                 ..
             }| {
                sdk::user::query_user::User {
                    id: id.into(),
                    user_source: to_sdk_user_source(user_source),
                    name: name.into(),
                    avatar_url: avatar_url.into(),
                    created_time: created_time.into(),
                    last_modified_time: last_modified_time.into(),
                }
            },
        )
        .collect();
    return Ok(PaginationList {
        pagination: pagination,
        list: list,
    });
}
