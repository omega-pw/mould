use super::super::job_record::merge_step_and_resource_record;
use super::merge_step_and_resource;
use super::start_run;
use super::Step;
use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::EnvironmentOpt;
use crate::model::environment_resource::EnvironmentResourceOpt;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::model::job::JobOpt;
use crate::model::job_record::enums::Status;
use crate::model::job_record::JobRecord;
use crate::model::job_step::JobStepOpt;
use crate::model::job_step_record::enums::Status as StepStatus;
use crate::model::job_step_record::JobStepRecord;
use crate::model::job_step_resource_record::enums::Status as StepResourceStatus;
use crate::model::job_step_resource_record::JobStepResourceRecord;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::EnvironmentResourceBaseService;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use crate::service::base::JobBaseService;
use crate::service::base::JobRecordBaseService;
use crate::service::base::JobStepBaseService;
use crate::service::base::JobStepRecordBaseService;
use crate::service::base::JobStepResourceRecordBaseService;
use chrono::Utc;
use sdk::job::start_job::StartJobReq;
use sdk::job::start_job::StartJobResp;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn start_job(
    org_id: Id,
    _user: User,
    start_job_req: StartJobReq,
) -> Result<StartJobResp, ErrNo> {
    let StartJobReq {
        job_id,
        environment_id,
    } = start_job_req;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_base_service = JobBaseService::new(&transaction);
    let job_step_base_service = JobStepBaseService::new(&transaction);
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let environment_resource_base_service = EnvironmentResourceBaseService::new(&transaction);
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    let job_record_base_service = JobRecordBaseService::new(&transaction);
    let job_step_record_base_service = JobStepRecordBaseService::new(&transaction);
    let job_step_resource_record_base_service = JobStepResourceRecordBaseService::new(&transaction);
    let params = JobOpt {
        org_id: Some(org_id),
        id: Some(job_id),
        ..JobOpt::empty()
    };
    let job_opt = job_base_service.query_job_one(&params).await?;
    let job =
        job_opt.ok_or_else(|| ErrNo::CommonError(LightString::from_static("该任务不存在")))?;
    let environment_opt = environment_base_service
        .query_environment_one(&EnvironmentOpt {
            org_id: Some(org_id),
            id: Some(environment_id),
            ..EnvironmentOpt::empty()
        })
        .await?;
    let environment = environment_opt
        .ok_or_else(|| ErrNo::CommonError(LightString::from_static("目标环境不存在")))?;

    //查询job的所有步骤
    let mut job_step_list = job_step_base_service
        .query_job_step_batch(&JobStepOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            ..JobStepOpt::empty()
        })
        .await?;
    job_step_list.sort_by_key(|item| item.seq);

    //查询环境的资源规格
    let environment_schema_resource_list = environment_schema_resource_base_service
        .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(environment.environment_schema_id.into()),
            ..EnvironmentSchemaResourceOpt::empty()
        })
        .await?;

    //查询环境所有的资源
    let environment_resource_list = environment_resource_base_service
        .query_environment_resource_batch(&EnvironmentResourceOpt {
            org_id: Some(org_id),
            environment_id: Some(environment_id.into()),
            ..EnvironmentResourceOpt::empty()
        })
        .await?;

    let job_step_list = merge_step_and_resource(
        &context,
        job_step_list,
        environment_schema_resource_list,
        environment_resource_list,
    )?;

    let record_id = context.new_id();
    let curr_time = Utc::now();
    let mut job_step_record_list = Vec::new();
    let mut job_step_resource_record_list = Vec::new();
    for step in job_step_list.into_iter() {
        match step {
            Step::Auto {
                job_step,
                extension_id,
                resource_list,
            } => {
                let job_step_record_id = context.new_id();
                let job_step_record = JobStepRecord {
                    id: job_step_record_id,
                    org_id: org_id,
                    job_id: job_id.into(),
                    environment_id: environment_id.into(),
                    record_id: record_id.into(),
                    job_step_id: job_step.id,
                    step_name: job_step.name,
                    step_type: crate::model::job_step_record::enums::StepType::Auto,
                    step_remark: job_step.remark,
                    job_step_seq: job_step.seq,
                    extension_id: extension_id,
                    operation_id: job_step.operation_id,
                    operation_name: job_step.operation_name,
                    operation_parameter: job_step.operation_parameter,
                    attachments: job_step.attachments,
                    status: StepStatus::Pending,
                    created_time: curr_time,
                    last_modified_time: curr_time,
                };
                job_step_record_list.push(job_step_record);
                for resource in resource_list {
                    let job_step_resource_record_id = context.new_id();
                    let job_step_resource_record = JobStepResourceRecord {
                        id: job_step_resource_record_id,
                        org_id: org_id,
                        job_id: job_id.into(),
                        environment_id: environment_id.into(),
                        record_id: record_id.into(),
                        job_step_record_id: job_step_record_id,
                        environment_resource_id: resource.id,
                        resource_name: resource.name,
                        extension_configuration: resource.extension_configuration,
                        output_file: None,
                        output_content: None,
                        status: StepResourceStatus::Pending,
                        created_time: curr_time,
                        last_modified_time: curr_time,
                    };
                    job_step_resource_record_list.push(job_step_resource_record);
                }
            }
            Step::Manual { job_step } => {
                //手动的步骤，仍然添加一条记录
                let job_step_record_id = context.new_id();
                let job_step_record = JobStepRecord {
                    id: job_step_record_id,
                    org_id: org_id,
                    job_id: job_id.into(),
                    environment_id: environment_id.into(),
                    record_id: record_id.into(),
                    job_step_id: job_step.id,
                    step_name: job_step.name,
                    step_type: crate::model::job_step_record::enums::StepType::Manual,
                    step_remark: job_step.remark,
                    job_step_seq: job_step.seq,
                    extension_id: String::from(""),
                    operation_id: String::from(""),
                    operation_name: String::from(""),
                    operation_parameter: String::from(""),
                    attachments: job_step.attachments,
                    status: StepStatus::Pending,
                    created_time: curr_time,
                    last_modified_time: curr_time,
                };
                job_step_record_list.push(job_step_record);
            }
        }
    }
    job_record_base_service
        .insert_job_record(&JobRecord {
            id: record_id,
            org_id: org_id,
            job_id: job.id,
            environment_id: environment_id.into(),
            status: Status::Running,
            created_time: curr_time,
            last_modified_time: curr_time,
        })
        .await?;
    job_step_record_base_service
        .insert_job_step_record_batch(&job_step_record_list)
        .await?;
    job_step_resource_record_base_service
        .insert_job_step_resource_record_batch(&job_step_resource_record_list)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    let step_record_list =
        merge_step_and_resource_record(job_step_record_list, job_step_resource_record_list);
    tokio::spawn(async move {
        if let Err(err) = start_run(context, step_record_list, record_id).await {
            log::error!("执行任务发生错误, {:?}", err);
        }
    });
    return Ok(PrimaryKey {
        id: record_id.into(),
    });
}
