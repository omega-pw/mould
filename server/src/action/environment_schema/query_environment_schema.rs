use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment_schema::EnvironmentSchema;
use crate::model::environment_schema::EnvironmentSchemaOpt;
use crate::sdk;
use crate::service::base::EnvironmentSchemaBaseService;
use crate::service::environment_schema::EnvironmentSchemaService;
use sdk::environment_schema::query_environment_schema::QueryEnvironmentSchemaReq;
use tihu::pagination::PaginationList;
use tihu::Id;
use tihu::Pagination;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn query_environment_schema(
    org_id: Id,
    _user: User,
    query_environment_schema_req: QueryEnvironmentSchemaReq,
) -> Result<PaginationList<sdk::environment_schema::EnvironmentSchema>, ErrNo> {
    let QueryEnvironmentSchemaReq {
        name,
        page_no,
        page_size,
    } = query_environment_schema_req;
    let params = EnvironmentSchemaOpt {
        org_id: Some(org_id),
        name: name.map(|v| v.into()),
        ..EnvironmentSchemaOpt::empty()
    };
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_schema_base_service = EnvironmentSchemaBaseService::new(&transaction);
    let environment_schema_service = EnvironmentSchemaService::new(&transaction);
    let count = environment_schema_base_service
        .query_environment_schema_count(&params)
        .await?;
    let pagination = Pagination::new(count, page_no.unwrap_or(1), page_size, None);
    let environment_schema_list = environment_schema_service
        .query_environment_schema(pagination.page_no, pagination.page_size, &params)
        .await?;
    let list = environment_schema_list
        .into_iter()
        .map(
            |EnvironmentSchema {
                 id,
                 name,
                 created_time,
                 last_modified_time,
                 ..
             }| {
                sdk::environment_schema::EnvironmentSchema {
                    id: id.into(),
                    name: name.into(),
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
