use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::EnvironmentOpt;
use crate::model::environment::EnvironmentProperty;
use crate::model::environment_resource::EnvironmentResource;
use crate::model::environment_resource::EnvironmentResourceOpt;
use crate::model::environment_resource::EnvironmentResourceProperty;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::native_common::utils::list;
use crate::native_common::utils::list::group_sub_list;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::EnvironmentResourceBaseService;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use chrono::Utc;
use sdk::environment::update_environment::UpdateEnvironmentReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn update_environment(
    org_id: Id,
    _user: User,
    update_environment_req: UpdateEnvironmentReq,
) -> Result<(), ErrNo> {
    let UpdateEnvironmentReq {
        id,
        name,
        schema_resource_list,
    } = update_environment_req;
    let environment_id = id;
    let context = get_context()?;
    for schema_resource in &schema_resource_list {
        let extension = context
            .get_extension(&schema_resource.extension_id)
            .ok_or_else(|| -> ErrNo {
                ErrNo::CommonError(LightString::from(format!(
                    "id为\"{}\"的扩展未找到!",
                    schema_resource.extension_id,
                )))
            })?;
        for resource in &schema_resource.resource_list {
            let extension_configuration =
                serde_json::from_str::<serde_json::Value>(&resource.extension_configuration)
                    .map_err(|err| -> ErrNo {
                        log::error!("扩展配置格式不正确：{}", err);
                        return ErrNo::CommonError(LightString::Static("扩展配置格式不正确"));
                    })?;
            extension
                .validate_configuration(extension_configuration)
                .map_err(|err| ErrNo::CommonError(err.into()))?;
        }
    }
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let params = EnvironmentOpt {
        org_id: Some(org_id),
        id: Some(environment_id),
        ..EnvironmentOpt::empty()
    };
    let environment_opt = environment_base_service
        .query_environment_one(&params)
        .await?;
    let environment = environment_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("待更新的环境不存在！"))
    })?;
    let environment_schema_id = environment.environment_schema_id;
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    let environment_resource_base_service = EnvironmentResourceBaseService::new(&transaction);

    //查询该环境应该有的资源
    let environment_schema_resource_list = environment_schema_resource_base_service
        .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(environment_schema_id.into()),
            ..EnvironmentSchemaResourceOpt::empty()
        })
        .await?;
    let environment_schema_resource_list = group_sub_list(
        environment_schema_resource_list,
        schema_resource_list,
        |existed, save| save.id == existed.id && save.extension_id == existed.extension_id,
    );
    let mut resource_list = Vec::new();
    let curr_time = Utc::now();
    for (environment_schema_resource, schema_resource_list) in environment_schema_resource_list {
        if schema_resource_list.is_empty() {
            return Err(ErrNo::CommonError(LightString::from(format!(
                "没有添加\"{}\"对应的资源",
                environment_schema_resource.name
            ))));
        } else {
            for schema_resource in schema_resource_list {
                if schema_resource.resource_list.is_empty() {
                    return Err(ErrNo::CommonError(LightString::from(format!(
                        "没有添加\"{}\"对应的资源",
                        environment_schema_resource.name
                    ))));
                } else {
                    for resource in schema_resource.resource_list {
                        resource_list.push((
                            resource,
                            environment_schema_resource.id,
                            environment_schema_resource.extension_id.clone(),
                            environment_schema_resource.extension_name.clone(),
                        ));
                    }
                }
            }
        }
    }
    let existed_resource_list = environment_resource_base_service
        .query_environment_resource_batch(&EnvironmentResourceOpt {
            org_id: Some(org_id),
            environment_id: Some(id.into()),
            ..EnvironmentResourceOpt::empty()
        })
        .await?;
    let action = list::group_list_action(
        resource_list,
        existed_resource_list,
        |(resource, schema_resource_id, extension_id, extension_name)| {
            if let Some(id) = resource.id {
                list::Either::Right((
                    id,
                    resource,
                    schema_resource_id,
                    extension_id,
                    extension_name,
                ))
            } else {
                let id = context.new_id();
                list::Either::Left(EnvironmentResource {
                    id: id,
                    org_id: org_id,
                    environment_id: environment_id.into(),
                    schema_resource_id: schema_resource_id,
                    name: resource.name,
                    extension_id: extension_id,
                    extension_name: extension_name,
                    extension_configuration: resource.extension_configuration,
                    created_time: curr_time,
                    last_modified_time: curr_time,
                })
            }
        },
        |(id, _resource, schema_resource_id, _extension_id, _extension_name), existed| {
            id == &existed.id && schema_resource_id == &existed.schema_resource_id
        },
        |(id, resource, schema_resource_id, _extension_id, _extension_name), existed| {
            id == &existed.id
                && schema_resource_id == &existed.schema_resource_id
                && resource.name == existed.name
                && resource.extension_configuration == existed.extension_configuration
        },
        |(_, resource, schema_resource_id, extension_id, extension_name)| {
            let id = context.new_id();
            EnvironmentResource {
                id: id,
                org_id: org_id,
                environment_id: environment_id.into(),
                schema_resource_id: schema_resource_id,
                name: resource.name,
                extension_id: extension_id,
                extension_name: extension_name,
                extension_configuration: resource.extension_configuration,
                created_time: curr_time,
                last_modified_time: curr_time,
            }
        },
    );

    let mut has_operation = false;
    if !action.add_list.is_empty() {
        has_operation = true;
        environment_resource_base_service
            .insert_environment_resource_batch(&action.add_list)
            .await?;
    }
    for (id, resource, _schema_resource_id, _extension_id, _extension_name) in action.update_list {
        has_operation = true;
        let changes: Vec<EnvironmentResourceProperty> = vec![
            EnvironmentResourceProperty::Name(resource.name),
            EnvironmentResourceProperty::ExtensionConfiguration(resource.extension_configuration),
            EnvironmentResourceProperty::LastModifiedTime(curr_time),
        ];
        environment_resource_base_service
            .update_environment_resource(id.into(), &changes)
            .await?;
    }
    if !action.remove_list.is_empty() {
        has_operation = true;
        let removed_ids: Vec<_> = action
            .remove_list
            .into_iter()
            .map(|resource| resource.id)
            .collect();
        environment_resource_base_service
            .delete_environment_resource_batch(&removed_ids)
            .await?;
    }
    let mut changes: Vec<EnvironmentProperty> = vec![EnvironmentProperty::Name(name.into())];
    changes.retain(|property| !environment.eq(property));
    if !changes.is_empty() {
        has_operation = true;
        changes.push(EnvironmentProperty::LastModifiedTime(curr_time));
        environment_base_service
            .update_environment(environment_id.into(), &changes)
            .await?;
    }
    if has_operation {
        transaction
            .commit()
            .await
            .map_err(commit_transaction_error)?;
    }
    return Ok(());
}
