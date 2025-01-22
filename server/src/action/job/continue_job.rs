use super::super::job_record::merge_step_and_resource_record;
use super::start_run;
use super::StepRecord;
use crate::get_context;
use crate::middleware::auth::User;
use crate::model::job_record::enums::Status as RecordStatus;
use crate::model::job_record::JobRecordProperty;
use crate::model::job_step_record::enums::Status;
use crate::model::job_step_record::JobStepRecordOpt;
use crate::model::job_step_record::JobStepRecordProperty;
use crate::model::job_step_resource_record::JobStepResourceRecordOpt;
use crate::sdk;
use crate::service::base::JobRecordBaseService;
use crate::service::base::JobStepRecordBaseService;
use crate::service::base::JobStepResourceRecordBaseService;
use chrono::Utc;
use sdk::job::continue_job::ContinueJobReq;
use sdk::job::continue_job::ContinueJobResp;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn continue_job(
    org_id: Id,
    _user: User,
    continue_job_req: ContinueJobReq,
) -> Result<ContinueJobResp, ErrNo> {
    let ContinueJobReq {
        record_id,
        step_record_id,
        success,
    } = continue_job_req;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_step_record_base_service = JobStepRecordBaseService::new(&transaction);
    let job_step_resource_record_base_service = JobStepResourceRecordBaseService::new(&transaction);
    //查询任务生成的执行记录
    let job_step_record_list = job_step_record_base_service
        .query_job_step_record_batch(&JobStepRecordOpt {
            org_id: Some(org_id),
            record_id: Some(record_id.into()),
            ..JobStepRecordOpt::empty()
        })
        .await?;
    //查询任务生成的执行资源记录
    let job_step_resource_record_list = job_step_resource_record_base_service
        .query_job_step_resource_record_batch(&JobStepResourceRecordOpt {
            org_id: Some(org_id),
            record_id: Some(record_id.into()),
            ..JobStepResourceRecordOpt::empty()
        })
        .await?;
    let job_step_list =
        merge_step_and_resource_record(job_step_record_list, job_step_resource_record_list);
    let mut found = false;
    let mut sub_job_step_list = Vec::new();
    for job_step in job_step_list {
        if found {
            sub_job_step_list.push(job_step);
        } else {
            match job_step {
                StepRecord::Manual { job_step_record } => {
                    if step_record_id == job_step_record.id {
                        if Status::Running == job_step_record.status {
                            found = true;
                            let curr_time = Utc::now();
                            let changes: Vec<JobStepRecordProperty> = vec![
                                JobStepRecordProperty::Status(if success {
                                    Status::Success
                                } else {
                                    Status::Failure
                                }),
                                JobStepRecordProperty::LastModifiedTime(curr_time),
                            ];
                            job_step_record_base_service
                                .update_job_step_record(job_step_record.id, &changes)
                                .await?;
                            if !success {
                                //如果手动操作任务是失败，则不再操作
                                break;
                            }
                        } else {
                            return Err(ErrNo::CommonError(LightString::from_static(
                                "不是进行中的步骤",
                            )));
                        }
                    }
                }
                _ => (),
            }
        }
    }
    if found {
        if !success {
            let job_record_base_service = JobRecordBaseService::new(&transaction);
            let curr_time = Utc::now();
            let changes: Vec<JobRecordProperty> = vec![
                JobRecordProperty::Status(RecordStatus::Failure),
                JobRecordProperty::LastModifiedTime(curr_time),
            ];
            job_record_base_service
                .update_job_record(record_id.into(), &changes)
                .await?;
        }
        transaction
            .commit()
            .await
            .map_err(commit_transaction_error)?;
        if !success {
            //如果手动操作任务是失败，则不再操作
            return Ok(());
        }
    }
    if !sub_job_step_list.is_empty() {
        //还有后续步骤就继续执行
        tokio::spawn(async move {
            if let Err(err) = start_run(context, sub_job_step_list, record_id).await {
                log::error!("执行任务发生错误, {:?}", err);
            }
        });
    }
    return Ok(());
}
