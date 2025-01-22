use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment_schema::EnvironmentSchema;
use crate::model::environment_schema::EnvironmentSchemaOpt;
use crate::model::environment_schema::EnvironmentSchemaProperty;
use crate::model::environment_schema_resource::EnvironmentSchemaResource;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceProperty;
use crate::native_common::utils::list;
use crate::sdk;
use crate::service::base::EnvironmentSchemaBaseService;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use chrono::Utc;
use sdk::environment_schema::save_environment_schema::SaveEnvironmentSchemaReq;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn save_environment_schema(
    org_id: Id,
    _user: User,
    save_environment_schema_req: SaveEnvironmentSchemaReq,
) -> Result<PrimaryKey, ErrNo> {
    let SaveEnvironmentSchemaReq {
        id,
        name,
        resource_list,
    } = save_environment_schema_req;
    let context = get_context()?;
    let curr_time = Utc::now();
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_schema_base_service = EnvironmentSchemaBaseService::new(&transaction);
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    let extensions = context.get_extensions();
    let mut resource_list_extension_name = Vec::with_capacity(resource_list.len());
    for schema_resource in resource_list {
        let extension = extensions
            .iter()
            .find(|extension| extension.id == schema_resource.extension_id)
            .ok_or_else(|| -> ErrNo {
                ErrNo::CommonError(LightString::from(format!(
                    "id为\"{}\"的扩展未找到!",
                    schema_resource.extension_id,
                )))
            })?;
        resource_list_extension_name.push((schema_resource, extension.name.clone()));
    }
    let resource_list = resource_list_extension_name;
    if let Some(environment_schema_id) = id {
        let params = EnvironmentSchemaOpt {
            org_id: Some(org_id),
            id: Some(environment_schema_id),
            ..EnvironmentSchemaOpt::empty()
        };
        let environment_schema_opt = environment_schema_base_service
            .query_environment_schema_one(&params)
            .await?;
        let environment_schema = environment_schema_opt.ok_or_else(|| -> ErrNo {
            ErrNo::CommonError(LightString::from_static("待更新的环境规格不存在！"))
        })?;

        //查询已有的资源
        let environment_schema_resource_list = environment_schema_resource_base_service
            .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
                org_id: Some(org_id),
                environment_schema_id: Some(environment_schema_id.into()),
                ..EnvironmentSchemaResourceOpt::empty()
            })
            .await?;
        let action = list::group_list_action(
            resource_list,
            environment_schema_resource_list,
            |(schema_resource, extension_name)| {
                if let Some(id) = schema_resource.id {
                    list::Either::Right((id, schema_resource, extension_name))
                } else {
                    let id = context.new_id();
                    list::Either::Left(EnvironmentSchemaResource {
                        id: id,
                        org_id: org_id,
                        environment_schema_id: environment_schema_id.into(),
                        name: schema_resource.name,
                        extension_id: schema_resource.extension_id,
                        extension_name: extension_name,
                        created_time: curr_time,
                        last_modified_time: curr_time,
                    })
                }
            },
            |(id, _, _), existed| *id == existed.id,
            |(id, schema_resource, _), existed| {
                *id == existed.id && schema_resource.name == existed.name
            },
            |(_, schema_resource, extension_name)| {
                let id = context.new_id();
                EnvironmentSchemaResource {
                    id: id,
                    org_id: org_id,
                    environment_schema_id: environment_schema_id.into(),
                    name: schema_resource.name,
                    extension_id: schema_resource.extension_id,
                    extension_name: extension_name,
                    created_time: curr_time,
                    last_modified_time: curr_time,
                }
            },
        );
        let mut has_operation = false;
        if !action.add_list.is_empty() {
            has_operation = true;
            environment_schema_resource_base_service
                .insert_environment_schema_resource_batch(&action.add_list)
                .await?;
        }
        for (id, schema_resource, _extension_name) in action.update_list {
            has_operation = true;
            let changes: Vec<EnvironmentSchemaResourceProperty> = vec![
                EnvironmentSchemaResourceProperty::Name(schema_resource.name),
                EnvironmentSchemaResourceProperty::LastModifiedTime(curr_time),
            ];
            environment_schema_resource_base_service
                .update_environment_schema_resource(id.into(), &changes)
                .await?;
        }
        if !action.remove_list.is_empty() {
            has_operation = true;
            let removed_ids: Vec<_> = action
                .remove_list
                .into_iter()
                .map(|schema_resource| schema_resource.id)
                .collect();
            environment_schema_resource_base_service
                .delete_environment_schema_resource_batch(&removed_ids)
                .await?;
        }
        let mut changes: Vec<EnvironmentSchemaProperty> =
            vec![EnvironmentSchemaProperty::Name(name.into())];
        changes.retain(|property| !environment_schema.eq(property));
        if !changes.is_empty() {
            has_operation = true;
            let curr_time = Utc::now();
            changes.push(EnvironmentSchemaProperty::LastModifiedTime(curr_time));
            environment_schema_base_service
                .update_environment_schema(environment_schema_id.into(), &changes)
                .await?;
        }
        if has_operation {
            transaction
                .commit()
                .await
                .map_err(commit_transaction_error)?;
        }
        return Ok(PrimaryKey {
            id: environment_schema_id.into(),
        });
    } else {
        let environment_schema_id = context.new_id();
        let environment_schema = EnvironmentSchema {
            id: environment_schema_id,
            org_id: org_id,
            name: name.into(),
            created_time: curr_time,
            last_modified_time: curr_time,
        };
        let environment_schema_resource_list: Vec<EnvironmentSchemaResource> = resource_list
            .into_iter()
            .map(
                |(schema_resource, extension_name)| -> EnvironmentSchemaResource {
                    let id = context.new_id();
                    return EnvironmentSchemaResource {
                        id: id,
                        org_id: org_id,
                        environment_schema_id: environment_schema_id,
                        name: schema_resource.name,
                        extension_id: schema_resource.extension_id,
                        extension_name: extension_name,
                        created_time: curr_time,
                        last_modified_time: curr_time,
                    };
                },
            )
            .collect();
        environment_schema_base_service
            .insert_environment_schema(&environment_schema)
            .await?;
        environment_schema_resource_base_service
            .insert_environment_schema_resource_batch(&environment_schema_resource_list)
            .await?;
        transaction
            .commit()
            .await
            .map_err(commit_transaction_error)?;
        return Ok(PrimaryKey {
            id: environment_schema_id.into(),
        });
    }
}
