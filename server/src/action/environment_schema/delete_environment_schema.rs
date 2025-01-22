use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::EnvironmentOpt;
use crate::model::environment_schema::EnvironmentSchemaOpt;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::EnvironmentSchemaBaseService;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use sdk::environment_schema::delete_environment_schema::DeleteEnvironmentSchemaReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::commit_transaction_error;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn delete_environment_schema(
    org_id: Id,
    _user: User,
    delete_environment_schema_req: DeleteEnvironmentSchemaReq,
) -> Result<(), ErrNo> {
    let DeleteEnvironmentSchemaReq { id } = delete_environment_schema_req;
    let environment_schema_id = id;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_schema_base_service = EnvironmentSchemaBaseService::new(&transaction);
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let params = EnvironmentSchemaOpt {
        org_id: Some(org_id),
        id: Some(environment_schema_id),
        ..EnvironmentSchemaOpt::empty()
    };
    let environment_schema_opt = environment_schema_base_service
        .query_environment_schema_one(&params)
        .await?;
    environment_schema_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("待删除的环境规格不存在！"))
    })?;
    let env_count = environment_base_service
        .query_environment_count(&EnvironmentOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(environment_schema_id.into()),
            ..EnvironmentOpt::empty()
        })
        .await?;
    if 0 < env_count {
        return Err(ErrNo::CommonError(LightString::from_static(
            "该环境规格已经存在相应的环境，不能删除！",
        )));
    }
    let schema_resource_list = environment_schema_resource_base_service
        .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(environment_schema_id.into()),
            ..EnvironmentSchemaResourceOpt::empty()
        })
        .await?;
    let schema_resource_ids: Vec<_> = schema_resource_list
        .into_iter()
        .map(|schema_resource| schema_resource.id)
        .collect();
    environment_schema_base_service
        .delete_environment_schema(id.into())
        .await?;
    environment_schema_resource_base_service
        .delete_environment_schema_resource_batch(&schema_resource_ids)
        .await?;
    transaction
        .commit()
        .await
        .map_err(commit_transaction_error)?;
    return Ok(());
}
