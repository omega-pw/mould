pub mod continue_job;
pub mod delete_job;
pub mod insert_job;
pub mod query_job;
pub mod read_job;
pub mod start_job;
pub mod update_job;
use super::job_record::StepRecord;
use crate::model::environment_resource::EnvironmentResource;
use crate::model::environment_schema_resource::EnvironmentSchemaResource;
use crate::model::job_record::enums::Status;
use crate::model::job_record::JobRecordProperty;
use crate::model::job_step::enums::StepType;
use crate::model::job_step::JobStep;
use crate::model::job_step_record::enums::Status as StepStatus;
use crate::model::job_step_record::JobStepRecordProperty;
use crate::model::job_step_resource_record::enums::Status as StepResourceStatus;
use crate::model::job_step_resource_record::JobStepResourceRecordProperty;
use crate::sdk;
use crate::service::base::JobRecordBaseService;
use crate::service::base::JobStepRecordBaseService;
use crate::service::base::JobStepResourceRecordBaseService;
use crate::Context;
use chrono::Utc;
use futures::future::join_all;
use mould_extension_sdk::AppendLog;
use sdk::job_record::read_job_record::LogLevel;
use sdk::job_record::read_job_record::StepResLog;
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;
use tokio::fs::read_to_string;
use uuid::Uuid;

/**
 * 环境资源
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    pub id: Id,                          //id
    pub name: String,                    //资源名称
    pub extension_id: String,            //扩展id
    pub extension_name: String,          //扩展名称
    pub extension_configuration: String, //扩展配置
}

async fn try_start_run(
    context: Arc<Context>,
    step_record_list: Vec<StepRecord>,
) -> Result<Status, ErrNo> {
    for step_record in step_record_list {
        match step_record {
            StepRecord::Auto {
                job_step_record,
                step_resource_record_list,
            } => {
                update_step_record(context.clone(), job_step_record.id, StepStatus::Running)
                    .await?;
                let context = context.clone();
                let extension_id: LightString = job_step_record.extension_id.into();
                let operation_id: LightString = job_step_record.operation_id.into();
                let operation_parameter: LightString = job_step_record.operation_parameter.into();
                let results = join_all(step_resource_record_list.into_iter().enumerate().map(
                    |(resource_index, step_resource_record)| {
                        call_extension(
                            context.clone(),
                            extension_id.clone(),
                            step_resource_record.extension_configuration.into(),
                            operation_id.clone(),
                            operation_parameter.clone(),
                            step_resource_record.id,
                            resource_index as u32,
                        )
                    },
                ))
                .await;
                for result in results {
                    if let Err(err) = result {
                        update_step_record(context, job_step_record.id, StepStatus::Failure)
                            .await?;
                        return Err(err);
                    }
                }
                update_step_record(context, job_step_record.id, StepStatus::Success).await?;
            }
            StepRecord::Manual { job_step_record } => {
                //手动任务，把步骤状态改成Running
                update_step_record(context, job_step_record.id, StepStatus::Running).await?;
                //全局任务返回Running
                return Ok(Status::Running);
            }
        }
    }
    return Ok(Status::Success);
}

async fn update_step_record(
    context: Arc<Context>,
    step_record_id: Id,
    step_status: StepStatus,
) -> Result<(), ErrNo> {
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let curr_time = Utc::now();
    let job_step_record_base_service = JobStepRecordBaseService::new(&transaction);
    let changes: Vec<JobStepRecordProperty> = vec![
        JobStepRecordProperty::Status(step_status),
        JobStepRecordProperty::LastModifiedTime(curr_time),
    ];
    job_step_record_base_service
        .update_job_step_record(step_record_id, &changes)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(());
}

enum Output {
    File(String),
    Content(String),
}

async fn update_step_resource_record(
    context: Arc<Context>,
    step_resource_record_id: Id,
    step_resource_status: StepResourceStatus,
    output: Output,
) -> Result<(), ErrNo> {
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let curr_time = Utc::now();
    let job_step_resource_record_base_service = JobStepResourceRecordBaseService::new(&transaction);
    let changes: Vec<JobStepResourceRecordProperty> = vec![
        JobStepResourceRecordProperty::Status(step_resource_status),
        match output {
            Output::File(file) => JobStepResourceRecordProperty::OutputFile(Some(file)),
            Output::Content(content) => JobStepResourceRecordProperty::OutputContent(Some(content)),
        },
        JobStepResourceRecordProperty::LastModifiedTime(curr_time),
    ];
    job_step_resource_record_base_service
        .update_job_step_resource_record(step_resource_record_id, &changes)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(());
}

async fn start_run(
    context: Arc<Context>,
    step_record_list: Vec<StepRecord>,
    record_id: Id,
) -> Result<(), ErrNo> {
    let result = try_start_run(context.clone(), step_record_list).await;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let curr_time = Utc::now();
    let job_record_base_service = JobRecordBaseService::new(&transaction);
    let changes: Vec<JobRecordProperty> = vec![
        JobRecordProperty::Status(match result {
            Ok(status) => status,
            Err(_) => Status::Failure,
        }),
        JobRecordProperty::LastModifiedTime(curr_time),
    ];
    job_record_base_service
        .update_job_record(record_id, &changes)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return result.map(|_| ());
}

async fn try_call_extension(
    context: Arc<Context>,
    extension_id: LightString,
    extension_configuration: LightString,
    operation_id: LightString,
    operation_parameter: LightString,
    resource_index: u32,
    append_log: AppendLog,
) -> Result<(), LightString> {
    let extension_configuration =
        serde_json::from_str(&extension_configuration).map_err(|err| err.to_string())?;
    let operation_parameter =
        serde_json::from_str(&operation_parameter).map_err(|err| err.to_string())?;
    let extension = context
        .get_extension(&extension_id)
        .ok_or_else(|| LightString::from(format!("没有找到id为{}的扩展!", extension_id)))?;
    return extension
        .handle(
            extension_configuration,
            &operation_id,
            operation_parameter,
            context.get_extension_context(),
            &append_log,
            resource_index,
        )
        .await
        .map_err(LightString::from);
}

fn try_write_log(log_file: &Mutex<File>, log: &StepResLog) -> Result<(), anyhow::Error> {
    let log_content = serde_json::to_string(log)?;
    let log = format!("{},", log_content);
    let mut log_file = log_file.lock().unwrap();
    log_file.write_all(log.as_bytes())?;
    log_file.flush()?;
    return Ok(());
}

async fn call_extension(
    context: Arc<Context>,
    extension_id: LightString,
    extension_configuration: LightString,
    operation_id: LightString,
    operation_parameter: LightString,
    step_resource_record_id: Id,
    resource_index: u32,
) -> Result<(), ErrNo> {
    let log_file_name = format!("{}.log", Uuid::new_v4().to_string());
    let log_file_path = format!("{}/{}", context.config.job_log_dir, log_file_name);
    let log_file = File::create(&log_file_path).map_err(|err| {
        ErrNo::CommonError(LightString::from(format!("创建任务日志文件报错：{}", err)))
    })?;
    let log_file = Mutex::new(log_file);
    update_step_resource_record(
        context.clone(),
        step_resource_record_id,
        StepResourceStatus::Running,
        Output::File(log_file_name),
    )
    .await?;
    let append_log: AppendLog = Arc::new(
        move |level: mould_extension_sdk::LogLevel, content: String| {
            let log = StepResLog {
                time: Utc::now(),
                level: match level {
                    mould_extension_sdk::LogLevel::Error => LogLevel::Error,
                    mould_extension_sdk::LogLevel::Warn => LogLevel::Warn,
                    mould_extension_sdk::LogLevel::Info => LogLevel::Info,
                    mould_extension_sdk::LogLevel::Debug => LogLevel::Debug,
                    mould_extension_sdk::LogLevel::Trace => LogLevel::Trace,
                },
                content: content,
            };
            if let Err(err) = try_write_log(&log_file, &log) {
                log::error!("写任务日志失败：{:?}", err);
            }
        },
    );
    let result = try_call_extension(
        context.clone(),
        extension_id,
        extension_configuration,
        operation_id,
        operation_parameter,
        resource_index,
        append_log,
    )
    .await;
    let output = read_to_string(&log_file_path).await.map_err(|err| {
        ErrNo::CommonError(LightString::from(format!("读取任务日志内容失败：{}", err)))
    })?;
    let mut output = format!("[{}", output);
    let status = match result.as_ref() {
        Ok(_) => StepResourceStatus::Success,
        Err(error) => {
            let log = StepResLog {
                time: Utc::now(),
                level: LogLevel::Error,
                content: error.to_string(),
            };
            let log_content = serde_json::to_string(&log).map_err(ErrNo::SerializeError)?;
            output.push_str(&log_content);
            StepResourceStatus::Failure
        }
    };
    if output.ends_with(",") {
        output.pop();
    }
    output.push_str("]");
    update_step_resource_record(
        context,
        step_resource_record_id,
        status,
        Output::Content(output),
    )
    .await?;
    if let Err(err) = remove_file(log_file_path) {
        log::error!("移除任务日志失败：{:?}", err);
    }
    return result.map(|_| ()).map_err(ErrNo::CommonError);
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Step {
    Auto {
        job_step: JobStep,
        extension_id: String,
        resource_list: Vec<Resource>,
    },
    Manual {
        job_step: JobStep,
    },
}

fn merge_step_and_resource(
    context: &Context,
    job_step_list: Vec<JobStep>,
    environment_schema_resource_list: Vec<EnvironmentSchemaResource>,
    environment_resource_list: Vec<EnvironmentResource>,
) -> Result<Vec<Step>, ErrNo> {
    let mut list = Vec::with_capacity(job_step_list.len());
    for job_step in job_step_list {
        match job_step.step_type {
            StepType::Auto => {
                let schema_resource = environment_schema_resource_list
                    .iter()
                    .find(|schema_resource| job_step.schema_resource_id == Some(schema_resource.id))
                    .ok_or_else(|| {
                        ErrNo::CommonError(LightString::from(format!(
                            "步骤\"{}\"未找到对应资源定义",
                            job_step.name
                        )))
                    })?;
                let extension_id = schema_resource.extension_id.clone();
                let (extension_info, extension) = context
                    .get_extension_info(&extension_id)
                    .ok_or_else(|| -> ErrNo {
                        ErrNo::CommonError(LightString::from(format!(
                            "扩展\"{}\"未找到!",
                            schema_resource.extension_name,
                        )))
                    })?;
                if extension_info
                    .operations
                    .iter()
                    .any(|operation| operation.id == job_step.operation_id)
                {
                    let operation_parameter =
                        serde_json::from_str::<serde_json::Value>(&job_step.operation_parameter)
                            .map_err(|err| -> ErrNo {
                                log::error!("操作参数格式不正确：{}", err);
                                return ErrNo::CommonError(LightString::Static(
                                    "操作参数格式不正确",
                                ));
                            })?;
                    extension
                        .validate_operation_parameter(&job_step.operation_id, operation_parameter)
                        .map_err(|err| ErrNo::CommonError(err.into()))?;
                } else {
                    return Err(ErrNo::CommonError(LightString::from(format!(
                        "扩展\"{}\"没有名为\"{}\"的操作!",
                        extension_info.name, job_step.operation_name
                    ))));
                }
                let resource_list: Vec<_> = environment_resource_list
                    .iter()
                    .filter(|env| job_step.schema_resource_id == Some(env.schema_resource_id))
                    .map(|environment_resource| Resource {
                        id: environment_resource.id,                                 //id
                        name: environment_resource.name.clone(),                     //资源名称
                        extension_id: environment_resource.extension_id.clone(),     //扩展id
                        extension_name: environment_resource.extension_name.clone(), //扩展名称
                        extension_configuration: environment_resource
                            .extension_configuration
                            .clone(), //扩展配置
                    })
                    .collect();
                if resource_list.is_empty() {
                    return Err(ErrNo::CommonError(LightString::from(format!(
                        "步骤\"{}\"没有匹配到任何资源",
                        job_step.name
                    ))));
                } else {
                    for resource in &resource_list {
                        if extension_id != resource.extension_id {
                            return Err(ErrNo::CommonError(LightString::from(format!(
                                "环境资源\"{}\"对应的扩展\"{}\"和\"{}\"不匹配",
                                resource.name, resource.extension_name, extension_info.name
                            ))));
                        }
                    }
                }
                list.push(Step::Auto {
                    job_step: job_step,
                    extension_id: extension_id,
                    resource_list: resource_list,
                });
            }
            StepType::Manual => {
                list.push(Step::Manual { job_step: job_step });
            }
        }
    }
    return Ok(list);
}
