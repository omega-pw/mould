use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::model::job::Job;
use crate::model::job_step::enums::StepType;
use crate::model::job_step::JobStep;
use crate::sdk;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use crate::service::base::JobBaseService;
use crate::service::base::JobStepBaseService;
use chrono::Utc;
use sdk::job::insert_job::InsertJobReq;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn insert_job(
    org_id: Id,
    _user: User,
    insert_job_req: InsertJobReq,
) -> Result<PrimaryKey, ErrNo> {
    let InsertJobReq {
        environment_schema_id,
        name,
        remark,
        job_step_list,
    } = insert_job_req;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_base_service = JobBaseService::new(&transaction);
    let job_step_base_service = JobStepBaseService::new(&transaction);
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    //查询任务对应的环境规格可以操作哪些资源
    let schema_resource_list = environment_schema_resource_base_service
        .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(environment_schema_id.into()),
            ..EnvironmentSchemaResourceOpt::empty()
        })
        .await?;
    let job_id = context.new_id();
    let curr_time = Utc::now();
    let job = Job {
        id: job_id,
        org_id: org_id,
        environment_schema_id: environment_schema_id.into(),
        name: name.into(),
        remark: remark.into(),
        created_time: curr_time,
        last_modified_time: curr_time,
    };
    let mut step_list: Vec<_> = Vec::with_capacity(job_step_list.len());
    for job_step in job_step_list {
        match job_step {
            sdk::job::insert_job::JobStep::Auto {
                name,
                schema_resource_id,
                operation_id,
                operation_parameter,
                remark,
                seq,
            } => {
                if let Some(schema_resource) = schema_resource_list
                    .iter()
                    .find(|schema_resource| schema_resource_id == schema_resource.id)
                {
                    let (extension_info, extension) = context
                        .get_extension_info(&schema_resource.extension_id)
                        .ok_or_else(|| -> ErrNo {
                            ErrNo::CommonError(LightString::from(format!(
                                "扩展\"{}\"未找到!",
                                schema_resource.extension_name,
                            )))
                        })?;
                    if let Some(operation) = extension_info
                        .operations
                        .iter()
                        .find(|operation| operation.id == operation_id)
                    {
                        let parameter =
                            serde_json::from_str::<serde_json::Value>(&operation_parameter)
                                .map_err(|err| -> ErrNo {
                                    log::error!("操作参数格式不正确：{}", err);
                                    return ErrNo::CommonError(LightString::Static(
                                        "操作参数格式不正确",
                                    ));
                                })?;
                        extension
                            .validate_operation_parameter(&operation_id, parameter)
                            .map_err(|err| ErrNo::CommonError(err.into()))?;
                        let id = context.new_id();
                        step_list.push(JobStep {
                            id: id, //步骤id
                            org_id: org_id,
                            job_id: job_id,                               //任务id
                            name: name,                                   //步骤名称
                            step_type: StepType::Auto,                    //步骤类型
                            schema_resource_id: Some(schema_resource_id), //环境规格资源id
                            operation_id: operation_id,                   //操作id
                            operation_name: operation.name.clone(),       //操作名称
                            operation_parameter: operation_parameter,     //操作参数
                            remark: remark,                               //备注
                            attachments: None,                            //附件
                            seq: seq,                                     //执行顺序
                            created_time: curr_time,                      //创建时间
                            last_modified_time: curr_time,                //更新时间
                        });
                    } else {
                        return Err(ErrNo::CommonError(LightString::from(format!(
                            "扩展\"{}\"没有id为\"{}\"的操作!",
                            schema_resource.extension_name, operation_id
                        ))));
                    }
                } else {
                    return Err(ErrNo::CommonError(LightString::from(format!(
                        "步骤\"{}\"所操作的环境资源不存在!",
                        name
                    ))));
                }
            }
            sdk::job::insert_job::JobStep::Manual {
                name,
                remark,
                attachments,
                seq,
            } => {
                let id = context.new_id();
                step_list.push(JobStep {
                    id: id, //步骤id
                    org_id: org_id,
                    job_id: job_id,                        //任务id
                    name: name,                            //步骤名称
                    step_type: StepType::Manual,           //步骤类型
                    schema_resource_id: None,              //环境规格资源id
                    operation_id: String::from(""),        //操作id
                    operation_name: String::from(""),      //操作名称
                    operation_parameter: String::from(""), //操作参数
                    remark: remark,                        //备注
                    attachments: attachments,              //附件
                    seq: seq,                              //执行顺序
                    created_time: curr_time,               //创建时间
                    last_modified_time: curr_time,         //更新时间
                });
            }
        }
    }
    job_base_service.insert_job(&job).await?;
    job_step_base_service
        .insert_job_step_batch(&step_list)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(PrimaryKey { id: job_id.into() });
}
