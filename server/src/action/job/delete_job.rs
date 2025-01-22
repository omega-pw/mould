use crate::get_context;
use crate::middleware::auth::User;
use crate::model::job::JobOpt;
use crate::model::job_record::enums::Status;
use crate::model::job_record::JobRecordOpt;
use crate::model::job_step::JobStepOpt;
use crate::model::job_step_record::JobStepRecordOpt;
use crate::model::job_step_resource_record::JobStepResourceRecordOpt;
use crate::sdk;
use crate::service::base::JobBaseService;
use crate::service::base::JobRecordBaseService;
use crate::service::base::JobStepBaseService;
use crate::service::base::JobStepRecordBaseService;
use crate::service::base::JobStepResourceRecordBaseService;
use sdk::job::delete_job::DeleteJobReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn delete_job(
    org_id: Id,
    _user: User,
    delete_job_req: DeleteJobReq,
) -> Result<(), ErrNo> {
    let DeleteJobReq { id } = delete_job_req;
    let job_id = id;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_base_service = JobBaseService::new(&transaction);
    let job_step_base_service = JobStepBaseService::new(&transaction);
    let job_record_base_service = JobRecordBaseService::new(&transaction);
    let job_step_record_base_service = JobStepRecordBaseService::new(&transaction);
    let job_step_resource_record_base_service = JobStepResourceRecordBaseService::new(&transaction);
    let params = JobOpt {
        org_id: Some(org_id),
        id: Some(job_id),
        ..JobOpt::empty()
    };
    let job_opt = job_base_service.query_job_one(&params).await?;
    job_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("待删除的任务不存在！"))
    })?;
    let running_job_count = job_record_base_service
        .query_job_record_count(&JobRecordOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            status: Some(Status::Running),
            ..JobRecordOpt::empty()
        })
        .await?;
    if 0 < running_job_count {
        return Err(ErrNo::CommonError(LightString::from_static(
            "该任务正在执行，不能删除！",
        )));
    }
    let job_step_list = job_step_base_service
        .query_job_step_batch(&JobStepOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            ..JobStepOpt::empty()
        })
        .await?;
    let job_record_list = job_record_base_service
        .query_job_record_batch(&JobRecordOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            ..JobRecordOpt::empty()
        })
        .await?;
    let job_step_record_list = job_step_record_base_service
        .query_job_step_record_batch(&JobStepRecordOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            ..JobStepRecordOpt::empty()
        })
        .await?;
    let job_step_resource_record_list = job_step_resource_record_base_service
        .query_job_step_resource_record_batch(&JobStepResourceRecordOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            ..JobStepResourceRecordOpt::empty()
        })
        .await?;
    let job_step_ids: Vec<_> = job_step_list
        .into_iter()
        .map(|job_step| job_step.id)
        .collect();
    let job_record_ids: Vec<_> = job_record_list
        .into_iter()
        .map(|job_record| job_record.id)
        .collect();
    let job_step_record_ids: Vec<_> = job_step_record_list
        .into_iter()
        .map(|job_step_record| job_step_record.id)
        .collect();
    let job_step_resource_record_ids: Vec<_> = job_step_resource_record_list
        .into_iter()
        .map(|job_step_resource_record| job_step_resource_record.id)
        .collect();
    job_base_service.delete_job(job_id.into()).await?;
    if !job_step_ids.is_empty() {
        job_step_base_service
            .delete_job_step_batch(&job_step_ids)
            .await?;
    }
    if !job_record_ids.is_empty() {
        job_record_base_service
            .delete_job_record_batch(&job_record_ids)
            .await?;
    }
    if !job_step_record_ids.is_empty() {
        job_step_record_base_service
            .delete_job_step_record_batch(&job_step_record_ids)
            .await?;
    }
    if !job_step_resource_record_ids.is_empty() {
        job_step_resource_record_base_service
            .delete_job_step_resource_record_batch(&job_step_resource_record_ids)
            .await?;
    }
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(());
}
