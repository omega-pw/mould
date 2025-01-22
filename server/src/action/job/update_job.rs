use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::model::job::JobOpt;
use crate::model::job::JobProperty;
use crate::model::job_step::enums::StepType;
use crate::model::job_step::JobStep;
use crate::model::job_step::JobStepOpt;
use crate::model::job_step::JobStepProperty;
use crate::native_common::utils::list;
use crate::native_common::utils::list::Either;
use crate::sdk;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use crate::service::base::JobBaseService;
use crate::service::base::JobStepBaseService;
use chrono::Utc;
use sdk::job::update_job::UpdateJobReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn update_job(
    org_id: Id,
    _user: User,
    update_job_req: UpdateJobReq,
) -> Result<(), ErrNo> {
    let UpdateJobReq {
        id,
        name,
        remark,
        job_step_list,
    } = update_job_req;
    let job_id = id;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let job_base_service = JobBaseService::new(&transaction);
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    let job_step_base_service = JobStepBaseService::new(&transaction);
    let params = JobOpt {
        org_id: Some(org_id),
        id: Some(job_id),
        ..JobOpt::empty()
    };
    let job_opt = job_base_service.query_job_one(&params).await?;
    let job = job_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("待更新的任务不存在！"))
    })?;
    //查询任务对应的环境规格可以操作哪些资源
    let schema_resource_list = environment_schema_resource_base_service
        .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(job.environment_schema_id),
            ..EnvironmentSchemaResourceOpt::empty()
        })
        .await?;
    let mut add_list: Vec<_> = Vec::new();
    let mut update_list: Vec<_> = Vec::new();
    let curr_time = Utc::now();
    for job_step in job_step_list {
        match job_step {
            sdk::job::update_job::JobStep::Auto {
                id,
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
                        if let Some(id) = id {
                            update_list.push((
                                id,
                                list::Either::Left((
                                    name,
                                    schema_resource_id,
                                    operation_id,
                                    operation.name.clone(),
                                    operation_parameter,
                                    remark,
                                    seq,
                                )),
                            ));
                        } else {
                            let id = context.new_id();
                            add_list.push(JobStep {
                                id: id, //步骤id
                                org_id: org_id,
                                job_id: job_id.into(),     //任务id
                                name: name,                //步骤名称
                                step_type: StepType::Auto, //步骤类型
                                schema_resource_id: Some(schema_resource_id), //环境规格资源id
                                operation_id: operation_id, //操作id
                                operation_name: operation.name.clone(), //操作名称
                                operation_parameter: operation_parameter, //操作参数
                                remark: remark,            //备注
                                attachments: None,         //附件
                                seq: seq,                  //执行顺序
                                created_time: curr_time,   //创建时间
                                last_modified_time: curr_time, //更新时间
                            });
                        }
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
            sdk::job::update_job::JobStep::Manual {
                id,
                name,
                remark,
                attachments,
                seq,
            } => {
                if let Some(id) = id {
                    update_list.push((id, list::Either::Right((name, remark, attachments, seq))));
                } else {
                    let id = context.new_id();
                    add_list.push(JobStep {
                        id: id, //步骤id
                        org_id: org_id,
                        job_id: job_id.into(),                 //任务id
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
    }

    //查询已有的步骤
    let existed_job_step_list = job_step_base_service
        .query_job_step_batch(&JobStepOpt {
            org_id: Some(org_id),
            job_id: Some(job_id.into()),
            ..JobStepOpt::empty()
        })
        .await?;
    let mut action = list::group_list_action(
        update_list,
        existed_job_step_list,
        |(step_id, step)| list::Either::Right((step_id, step)),
        |(id, step), existed| {
            id == &existed.id
                && match step {
                    Either::Left((
                        _name,
                        schema_resource_id,
                        operation_id,
                        _operation_name,
                        _operation_parameter,
                        _remark,
                        _seq,
                    )) => {
                        StepType::Auto == existed.step_type
                            && Some(schema_resource_id) == existed.schema_resource_id.as_ref()
                            && operation_id == &existed.operation_id
                    }
                    Either::Right((_name, _remark, _attachments, _seq)) => {
                        StepType::Manual == existed.step_type
                    }
                }
        },
        |(id, step), existed| {
            id == &existed.id
                && match step {
                    Either::Left((
                        name,
                        schema_resource_id,
                        operation_id,
                        operation_name,
                        operation_parameter,
                        remark,
                        seq,
                    )) => {
                        StepType::Auto == existed.step_type
                            && Some(schema_resource_id) == existed.schema_resource_id.as_ref()
                            && operation_id == &existed.operation_id
                            && name == &existed.name
                            && operation_name == &existed.operation_name
                            && operation_parameter == &existed.operation_parameter
                            && remark == &existed.remark
                            && seq == &existed.seq
                    }
                    Either::Right((name, remark, attachments, seq)) => {
                        StepType::Manual == existed.step_type
                            && name == &existed.name
                            && remark == &existed.remark
                            && attachments == &existed.attachments
                            && seq == &existed.seq
                    }
                }
        },
        |(_, step)| {
            let id = context.new_id();
            match step {
                Either::Left((
                    name,
                    schema_resource_id,
                    operation_id,
                    operation_name,
                    operation_parameter,
                    remark,
                    seq,
                )) => {
                    JobStep {
                        id: id, //步骤id
                        org_id: org_id,
                        job_id: job_id.into(),                        //任务id
                        name: name,                                   //步骤名称
                        step_type: StepType::Auto,                    //步骤类型
                        schema_resource_id: Some(schema_resource_id), //环境规格资源id
                        operation_id: operation_id,                   //操作id
                        operation_name: operation_name,               //操作名称
                        operation_parameter: operation_parameter,     //操作参数
                        remark: remark,                               //备注
                        attachments: None,                            //附件
                        seq: seq,                                     //执行顺序
                        created_time: curr_time,                      //创建时间
                        last_modified_time: curr_time,                //更新时间
                    }
                }
                Either::Right((name, remark, attachments, seq)) => {
                    JobStep {
                        id: id, //步骤id
                        org_id: org_id,
                        job_id: job_id.into(),                 //任务id
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
                    }
                }
            }
        },
    );
    action.add_list.append(&mut add_list);

    let mut has_operation = false;
    if !action.add_list.is_empty() {
        has_operation = true;
        job_step_base_service
            .insert_job_step_batch(&action.add_list)
            .await?;
    }
    for (id, step) in action.update_list {
        has_operation = true;
        let changes: Vec<JobStepProperty> = match step {
            Either::Left((
                name,
                _schema_resource_id,
                _operation_id,
                _operation_name,
                operation_parameter,
                remark,
                seq,
            )) => {
                vec![
                    JobStepProperty::Name(name),
                    JobStepProperty::OperationParameter(operation_parameter),
                    JobStepProperty::Remark(remark),
                    JobStepProperty::Seq(seq),
                    JobStepProperty::LastModifiedTime(curr_time),
                ]
            }
            Either::Right((name, remark, attachments, seq)) => {
                vec![
                    JobStepProperty::Name(name),
                    JobStepProperty::Remark(remark),
                    JobStepProperty::Attachments(attachments),
                    JobStepProperty::Seq(seq),
                    JobStepProperty::LastModifiedTime(curr_time),
                ]
            }
        };
        job_step_base_service
            .update_job_step(id.into(), &changes)
            .await?;
    }
    if !action.remove_list.is_empty() {
        has_operation = true;
        let removed_ids: Vec<_> = action.remove_list.into_iter().map(|step| step.id).collect();
        job_step_base_service
            .delete_job_step_batch(&removed_ids)
            .await?;
    }
    let mut changes: Vec<JobProperty> = vec![
        JobProperty::Name(name.into()),
        JobProperty::Remark(remark.into()),
    ];
    changes.retain(|property| !job.eq(property));
    if !changes.is_empty() {
        has_operation = true;
        changes.push(JobProperty::LastModifiedTime(curr_time));
        job_base_service.update_job(job_id.into(), &changes).await?;
    }
    if has_operation {
        transaction
            .commit()
            .await
            .map_err(commit_transaction_error)?;
    }
    return Ok(());
}
