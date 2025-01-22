use crate::get_context;
use crate::middleware::auth::User;
use crate::model::job::JobOpt;
use crate::sdk;
use crate::service::base::EnvironmentSchemaBaseService;
use crate::service::base::JobBaseService;
use crate::service::job::JobService;
use sdk::job::query_job::Job;
use sdk::job::query_job::QueryJobReq;
use std::collections::HashMap;
use std::collections::HashSet;
use tihu::pagination::PaginationList;
use tihu::Id;
use tihu::Pagination;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn query_job(
    org_id: Id,
    _user: User,
    query_job_req: QueryJobReq,
) -> Result<PaginationList<Job>, ErrNo> {
    let QueryJobReq {
        name,
        page_no,
        page_size,
    } = query_job_req;
    let params = JobOpt {
        org_id: Some(org_id),
        name: name.map(|v| v.into()),
        ..JobOpt::empty()
    };
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_base_service = JobBaseService::new(&transaction);
    let job_service = JobService::new(&transaction);
    let environment_schema_base_service = EnvironmentSchemaBaseService::new(&transaction);
    let count = job_base_service.query_job_count(&params).await?;
    let pagination = Pagination::new(count, page_no.unwrap_or(1), page_size, None);
    let job_list = job_service
        .query_job(pagination.page_no, pagination.page_size, &params)
        .await?;
    let mut list = Vec::with_capacity(job_list.len());
    if !job_list.is_empty() {
        let environment_schema_ids: HashSet<_> = job_list
            .iter()
            .map(|job| job.environment_schema_id)
            .collect();
        let environment_schema_ids: Vec<_> = environment_schema_ids.into_iter().collect();
        let environment_schema_list = environment_schema_base_service
            .read_environment_schema_batch(&environment_schema_ids)
            .await?;
        let environment_schema_map: HashMap<_, _> = environment_schema_list
            .into_iter()
            .map(|environment_schema| (environment_schema.id, environment_schema.name))
            .collect();
        for job in job_list {
            list.push(Job {
                id: job.id.into(),
                environment_schema_id: job.environment_schema_id.into(),
                environment_schema_name: environment_schema_map
                    .get(&job.environment_schema_id)
                    .map(|name| name.clone())
                    .unwrap_or_default(),
                name: job.name.into(),
                remark: job.remark.into(),
                created_time: job.created_time.into(),
                last_modified_time: job.last_modified_time.into(),
            })
        }
    }
    return Ok(PaginationList {
        pagination: pagination,
        list: list,
    });
}
