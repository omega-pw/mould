use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::EnvironmentOpt;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::EnvironmentSchemaBaseService;
use crate::service::environment::EnvironmentService;
use sdk::environment::query_environment::Environment;
use sdk::environment::query_environment::QueryEnvironmentReq;
use std::collections::HashMap;
use std::collections::HashSet;
use tihu::pagination::PaginationList;
use tihu::Id;
use tihu::Pagination;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn query_environment(
    org_id: Id,
    _user: User,
    query_environment_req: QueryEnvironmentReq,
) -> Result<PaginationList<Environment>, ErrNo> {
    let QueryEnvironmentReq {
        environment_schema_id,
        name,
        page_no,
        page_size,
    } = query_environment_req;
    let params = EnvironmentOpt {
        org_id: Some(org_id),
        environment_schema_id: environment_schema_id.map(|v| v.into()),
        name: name,
        ..EnvironmentOpt::empty()
    };
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let environment_service = EnvironmentService::new(&transaction);
    let environment_schema_base_service = EnvironmentSchemaBaseService::new(&transaction);
    let count = environment_base_service
        .query_environment_count(&params)
        .await?;
    let pagination = Pagination::new(count, page_no.unwrap_or(1), page_size, None);
    let environment_list = environment_service
        .query_environment(pagination.page_no, pagination.page_size, &params)
        .await?;
    let mut list = Vec::with_capacity(environment_list.len());
    if !environment_list.is_empty() {
        let environment_schema_ids: HashSet<_> = environment_list
            .iter()
            .map(|environment| environment.environment_schema_id)
            .collect();
        let environment_schema_ids: Vec<_> = environment_schema_ids.into_iter().collect();
        let environment_schema_list = environment_schema_base_service
            .read_environment_schema_batch(&environment_schema_ids)
            .await?;
        let environment_schema_map: HashMap<_, _> = environment_schema_list
            .into_iter()
            .map(|environment_schema| (environment_schema.id, environment_schema.name))
            .collect();
        for environment in environment_list {
            list.push(Environment {
                id: environment.id.into(),
                environment_schema_id: environment.environment_schema_id.into(),
                environment_schema_name: environment_schema_map
                    .get(&environment.environment_schema_id)
                    .map(|name| name.clone())
                    .unwrap_or_default(),
                name: environment.name.into(),
                created_time: environment.created_time.into(),
                last_modified_time: environment.last_modified_time.into(),
            })
        }
    };
    return Ok(PaginationList {
        pagination: pagination,
        list: list,
    });
}
