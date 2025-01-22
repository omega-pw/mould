use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::EnvironmentOpt;
use crate::model::environment_resource::EnvironmentResourceOpt;
use crate::model::job_record::enums::Status;
use crate::model::job_record::JobRecordOpt;
use crate::model::job_step_record::JobStepRecordOpt;
use crate::model::job_step_resource_record::JobStepResourceRecordOpt;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::EnvironmentResourceBaseService;
use crate::service::base::JobRecordBaseService;
use crate::service::base::JobStepRecordBaseService;
use crate::service::base::JobStepResourceRecordBaseService;
use sdk::environment::delete_environment::DeleteEnvironmentReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn delete_environment(
    org_id: Id,
    _user: User,
    delete_environment_req: DeleteEnvironmentReq,
) -> Result<(), ErrNo> {
    let DeleteEnvironmentReq { id } = delete_environment_req;
    let environment_id = id;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let environment_resource_base_service = EnvironmentResourceBaseService::new(&transaction);
    let job_record_base_service = JobRecordBaseService::new(&transaction);
    let job_step_record_base_service = JobStepRecordBaseService::new(&transaction);
    let job_step_resource_record_base_service = JobStepResourceRecordBaseService::new(&transaction);
    let params = EnvironmentOpt {
        org_id: Some(org_id),
        id: Some(environment_id),
        ..EnvironmentOpt::empty()
    };
    let environment_opt = environment_base_service
        .query_environment_one(&params)
        .await?;
    environment_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("待删除的环境不存在！"))
    })?;
    let running_job_count = job_record_base_service
        .query_job_record_count(&JobRecordOpt {
            org_id: Some(org_id),
            environment_id: Some(environment_id.into()),
            status: Some(Status::Running),
            ..JobRecordOpt::empty()
        })
        .await?;
    if 0 < running_job_count {
        return Err(ErrNo::CommonError(LightString::from_static(
            "该环境正在执行更新任务，不能删除！",
        )));
    }
    let resource_list = environment_resource_base_service
        .query_environment_resource_batch(&EnvironmentResourceOpt {
            org_id: Some(org_id),
            environment_id: Some(environment_id.into()),
            ..EnvironmentResourceOpt::empty()
        })
        .await?;
    let job_record_list = job_record_base_service
        .query_job_record_batch(&JobRecordOpt {
            org_id: Some(org_id),
            environment_id: Some(environment_id.into()),
            ..JobRecordOpt::empty()
        })
        .await?;
    let job_step_record_list = job_step_record_base_service
        .query_job_step_record_batch(&JobStepRecordOpt {
            org_id: Some(org_id),
            environment_id: Some(environment_id.into()),
            ..JobStepRecordOpt::empty()
        })
        .await?;
    let job_step_resource_record_list = job_step_resource_record_base_service
        .query_job_step_resource_record_batch(&JobStepResourceRecordOpt {
            org_id: Some(org_id),
            environment_id: Some(environment_id.into()),
            ..JobStepResourceRecordOpt::empty()
        })
        .await?;
    let resource_ids: Vec<_> = resource_list
        .into_iter()
        .map(|resource| resource.id)
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
    environment_base_service
        .delete_environment(id.into())
        .await?;
    if !resource_ids.is_empty() {
        environment_resource_base_service
            .delete_environment_resource_batch(&resource_ids)
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
