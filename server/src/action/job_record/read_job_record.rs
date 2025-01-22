use super::merge_step_and_resource_record;
use super::to_sdk_record_status;
use super::to_sdk_step_record_status;
use super::to_sdk_step_resource_record_status;
use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::EnvironmentOpt;
use crate::model::job::JobOpt;
use crate::model::job_record::JobRecordOpt;
use crate::model::job_step_record::JobStepRecordOpt;
use crate::model::job_step_resource_record::JobStepResourceRecordOpt;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::JobBaseService;
use crate::service::base::JobRecordBaseService;
use crate::service::base::JobStepRecordBaseService;
use crate::service::base::JobStepResourceRecordBaseService;
use sdk::job_record::enums::StepType;
use sdk::job_record::read_job_record::JobRecord;
use sdk::job_record::read_job_record::JobStepRecord;
use sdk::job_record::read_job_record::JobStepResourceRecord;
use sdk::job_record::read_job_record::ReadJobRecordReq;
use sdk::job_record::read_job_record::ReadJobRecordResp;
use sdk::job_record::read_job_record::StepRecord;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

fn to_sdk_step_type(val: crate::model::job_step_record::enums::StepType) -> StepType {
    match val {
        crate::model::job_step_record::enums::StepType::Auto => StepType::Auto,
        crate::model::job_step_record::enums::StepType::Manual => StepType::Manual,
    }
}

fn to_sdk_step_record(step_record: crate::model::job_step_record::JobStepRecord) -> JobStepRecord {
    JobStepRecord {
        id: step_record.id.into(),
        record_id: step_record.record_id.into(),
        job_step_id: step_record.job_step_id.into(),
        step_name: step_record.step_name,
        step_type: to_sdk_step_type(step_record.step_type),
        step_remark: step_record.step_remark,
        extension_id: step_record.extension_id,
        operation_id: step_record.operation_id,
        operation_parameter: step_record.operation_parameter,
        attachments: step_record.attachments,
        job_step_seq: step_record.job_step_seq,
        status: to_sdk_step_record_status(step_record.status),
        created_time: step_record.created_time,
        last_modified_time: step_record.last_modified_time,
    }
}

fn to_sdk_step_resource_record(
    step_resource_record: crate::model::job_step_resource_record::JobStepResourceRecord,
) -> JobStepResourceRecord {
    JobStepResourceRecord {
        id: step_resource_record.id.into(),
        record_id: step_resource_record.record_id.into(),
        job_step_record_id: step_resource_record.job_step_record_id.into(),
        environment_resource_id: step_resource_record.environment_resource_id.into(),
        resource_name: step_resource_record.resource_name,
        extension_configuration: step_resource_record.extension_configuration,
        output: step_resource_record.output_content,
        status: to_sdk_step_resource_record_status(step_resource_record.status),
        created_time: step_resource_record.created_time,
        last_modified_time: step_resource_record.last_modified_time,
    }
}

pub async fn read_job_record(
    org_id: Id,
    _user: User,
    read_job_record_req: ReadJobRecordReq,
) -> Result<ReadJobRecordResp, ErrNo> {
    let ReadJobRecordReq { id } = read_job_record_req;
    let record_id = id;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_record_base_service = JobRecordBaseService::new(&transaction);
    let job_step_record_base_service = JobStepRecordBaseService::new(&transaction);
    let job_step_resource_record_base_service = JobStepResourceRecordBaseService::new(&transaction);
    let job_base_service = JobBaseService::new(&transaction);
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let params = JobRecordOpt {
        org_id: Some(org_id),
        id: Some(record_id),
        ..JobRecordOpt::empty()
    };
    let job_record_opt = job_record_base_service
        .query_job_record_one(&params)
        .await?;
    let job_record = job_record_opt
        .ok_or_else(|| ErrNo::CommonError(LightString::from_static("该任务执行记录不存在")))?;

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
    let job_opt = job_base_service
        .query_job_one(&JobOpt {
            org_id: Some(org_id),
            id: Some(job_record.job_id),
            ..JobOpt::empty()
        })
        .await?;
    let environment_opt = environment_base_service
        .query_environment_one(&EnvironmentOpt {
            org_id: Some(org_id),
            id: Some(job_record.environment_id),
            ..EnvironmentOpt::empty()
        })
        .await?;
    let job_step_list =
        merge_step_and_resource_record(job_step_record_list, job_step_resource_record_list);
    let mut step_record_list = Vec::with_capacity(job_step_list.len());
    for step_record in job_step_list {
        match step_record {
            super::StepRecord::Auto {
                job_step_record,
                step_resource_record_list,
            } => {
                let mut list = Vec::with_capacity(step_resource_record_list.len());
                for mut step_resource_record in step_resource_record_list {
                    if step_resource_record.output_content.is_none() {
                        if let Some(output_file) = step_resource_record.output_file.as_ref() {
                            let log_file_path =
                                format!("{}/{}", context.config.job_log_dir, output_file);
                            if let Ok(mut content) = tokio::fs::read_to_string(log_file_path).await
                            {
                                if content.ends_with(",") {
                                    content.pop();
                                }
                                step_resource_record
                                    .output_content
                                    .replace(format!("[{}]", content));
                            }
                        }
                    }
                    list.push(to_sdk_step_resource_record(step_resource_record));
                }
                step_record_list.push(StepRecord::Auto {
                    job_step_record: to_sdk_step_record(job_step_record),
                    step_resource_record_list: list,
                });
            }
            super::StepRecord::Manual { job_step_record } => {
                step_record_list.push(StepRecord::Manual {
                    job_step_record: to_sdk_step_record(job_step_record),
                });
            }
        }
    }
    let job_record = JobRecord {
        id: job_record.id.into(),
        job_id: job_record.job_id.into(),
        job_name: job_opt.map(|job| job.name),
        environment_id: job_record.environment_id.into(),
        environment_name: environment_opt.map(|environment| environment.name),
        status: to_sdk_record_status(job_record.status),
        step_record_list: step_record_list,
        created_time: job_record.created_time,
        last_modified_time: job_record.last_modified_time,
    };
    return Ok(job_record);
}
