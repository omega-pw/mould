use crate::get_context;
use crate::middleware::auth::User;
use crate::model::job::JobOpt;
use crate::model::job_step::enums::StepType;
use crate::model::job_step::JobStepOpt;
use crate::sdk;
use crate::service::base::JobBaseService;
use crate::service::base::JobStepBaseService;
use sdk::job::read_job::Job;
use sdk::job::read_job::JobStep;
use sdk::job::read_job::ReadJobReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn read_job(org_id: Id, _user: User, read_job_req: ReadJobReq) -> Result<Job, ErrNo> {
    let ReadJobReq { id } = read_job_req;
    let job_id = id;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_base_service = JobBaseService::new(&transaction);
    let job_step_base_service = JobStepBaseService::new(&transaction);
    let params = JobOpt {
        org_id: Some(org_id),
        id: Some(job_id),
        ..JobOpt::empty()
    };
    let job_opt = job_base_service.query_job_one(&params).await?;
    let job = job_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("该任务不存在！"))
    })?;
    let mut job_step_list = job_step_base_service
        .query_job_step_batch(&JobStepOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            ..JobStepOpt::empty()
        })
        .await?;
    job_step_list.sort_by_key(|job_step| job_step.seq);
    let mut step_list = Vec::with_capacity(job_step_list.len());
    for job_step in job_step_list {
        step_list.push(match job_step.step_type {
            StepType::Auto => JobStep::Auto {
                id: job_step.id.into(),
                name: job_step.name,
                schema_resource_id: job_step.schema_resource_id.ok_or_else(|| {
                    ErrNo::CommonError(LightString::from(
                        "自动步骤的\"schema_resource_id\"字段缺失!",
                    ))
                })?,
                operation_id: job_step.operation_id,
                operation_name: job_step.operation_name,
                operation_parameter: job_step.operation_parameter,
                remark: job_step.remark,
                seq: job_step.seq,
            },
            StepType::Manual => JobStep::Manual {
                id: job_step.id.into(),
                name: job_step.name,
                remark: job_step.remark,
                attachments: job_step.attachments,
                seq: job_step.seq,
            },
        });
    }
    return Ok(Job {
        id: job.id.into(),
        environment_schema_id: job.environment_schema_id.into(),
        name: job.name,
        remark: job.remark,
        job_step_list: step_list,
    });
}
