use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::Environment;
use crate::model::environment_resource::EnvironmentResource;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::native_common::utils::list::group_sub_list;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::EnvironmentResourceBaseService;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use chrono::Utc;
use sdk::environment::insert_environment::InsertEnvironmentReq;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn insert_environment(
    org_id: Id,
    _user: User,
    insert_environment_req: InsertEnvironmentReq,
) -> Result<PrimaryKey, ErrNo> {
    let InsertEnvironmentReq {
        environment_schema_id,
        name,
        schema_resource_list,
    } = insert_environment_req;
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
    let environment_id = context.new_id();
    let curr_time = Utc::now();
    let environment = Environment {
        id: environment_id,
        org_id: org_id,
        environment_schema_id: environment_schema_id.into(),
        name: name.into(),
        created_time: curr_time,
        last_modified_time: curr_time,
    };
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_base_service = EnvironmentBaseService::new(&transaction);
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
                        let id = context.new_id();
                        resource_list.push(EnvironmentResource {
                            id: id,
                            org_id: org_id,
                            environment_id: environment_id,
                            schema_resource_id: environment_schema_resource.id,
                            name: resource.name,
                            extension_id: environment_schema_resource.extension_id.clone(),
                            extension_name: environment_schema_resource.extension_name.clone(),
                            extension_configuration: resource.extension_configuration,
                            created_time: curr_time,
                            last_modified_time: curr_time,
                        });
                    }
                }
            }
        }
    }
    environment_base_service
        .insert_environment(&environment)
        .await?;
    environment_resource_base_service
        .insert_environment_resource_batch(&resource_list)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(PrimaryKey {
        id: environment_id.into(),
    });
}
