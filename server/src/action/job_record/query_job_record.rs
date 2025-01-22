use crate::get_context;
use crate::middleware::auth::User;
use crate::model::job_record::enums::Status;
use crate::model::job_record::JobRecord;
use crate::model::job_record::JobRecordOpt;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::JobBaseService;
use crate::service::base::JobRecordBaseService;
use crate::service::job_record::JobRecordService;
use sdk::job_record::query_job_record::QueryJobRecordReq;
use std::collections::HashMap;
use std::collections::HashSet;
use tihu::pagination::PaginationList;
use tihu::Id;
use tihu::Pagination;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

fn from_sdk_status(val: sdk::job_record::enums::RecordStatus) -> Status {
    match val {
        sdk::job_record::enums::RecordStatus::Running => Status::Running,
        sdk::job_record::enums::RecordStatus::Success => Status::Success,
        sdk::job_record::enums::RecordStatus::Failure => Status::Failure,
    }
}
fn to_sdk_status(val: Status) -> sdk::job_record::enums::RecordStatus {
    match val {
        Status::Running => sdk::job_record::enums::RecordStatus::Running,
        Status::Success => sdk::job_record::enums::RecordStatus::Success,
        Status::Failure => sdk::job_record::enums::RecordStatus::Failure,
    }
}

pub async fn query_job_record(
    org_id: Id,
    _user: User,
    query_job_record_req: QueryJobRecordReq,
) -> Result<PaginationList<sdk::job_record::query_job_record::JobRecord>, ErrNo> {
    let QueryJobRecordReq {
        job_id,
        environment_id,
        status,
        page_no,
        page_size,
    } = query_job_record_req;
    let params = JobRecordOpt {
        org_id: Some(org_id),
        job_id: job_id.map(|v| v.into()),
        environment_id: environment_id.map(|v| v.into()),
        status: status.map(from_sdk_status),
        ..JobRecordOpt::empty()
    };
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_record_base_service = JobRecordBaseService::new(&transaction);
    let job_record_service = JobRecordService::new(&transaction);
    let job_base_service = JobBaseService::new(&transaction);
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let count = job_record_base_service
        .query_job_record_count(&params)
        .await?;
    let pagination = Pagination::new(count, page_no.unwrap_or(1), page_size, None);
    let job_record_list = job_record_service
        .query_job_record(pagination.page_no, pagination.page_size, &params)
        .await?;
    let job_ids: HashSet<_> = job_record_list
        .iter()
        .map(|job_record| job_record.job_id)
        .collect();
    let job_ids: Vec<_> = job_ids.into_iter().collect();
    let job_list = job_base_service.read_job_batch(&job_ids).await?;
    let job_map: HashMap<_, _> = job_list.into_iter().map(|job| (job.id, job.name)).collect();
    let environment_ids: HashSet<_> = job_record_list
        .iter()
        .map(|job_record| job_record.environment_id)
        .collect();
    let environment_ids: Vec<_> = environment_ids.into_iter().collect();
    let environment_list = environment_base_service
        .read_environment_batch(&environment_ids)
        .await?;
    let environment_map: HashMap<_, _> = environment_list
        .into_iter()
        .map(|environment| (environment.id, environment.name))
        .collect();
    let list = job_record_list
        .into_iter()
        .map(
            |JobRecord {
                 id,
                 job_id,
                 environment_id,
                 status,
                 created_time,
                 last_modified_time,
                 ..
             }| {
                sdk::job_record::query_job_record::JobRecord {
                    id: id.into(),
                    job_id: job_id.into(),
                    job_name: job_map.get(&job_id).map(|name| name.clone()),
                    environment_id: environment_id.into(),
                    environment_name: environment_map
                        .get(&environment_id)
                        .map(|name| name.clone()),
                    status: to_sdk_status(status),
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
