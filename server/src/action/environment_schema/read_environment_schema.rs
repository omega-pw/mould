use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment_schema::EnvironmentSchemaOpt;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::sdk;
use crate::service::base::EnvironmentSchemaBaseService;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::environment_schema::read_environment_schema::SchemaResource;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn read_environment_schema(
    org_id: Id,
    _user: User,
    read_environment_schema_req: ReadEnvironmentSchemaReq,
) -> Result<EnvironmentSchema, ErrNo> {
    let ReadEnvironmentSchemaReq { id } = read_environment_schema_req;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_schema_base_service = EnvironmentSchemaBaseService::new(&transaction);
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    let params = EnvironmentSchemaOpt {
        org_id: Some(org_id),
        id: Some(id),
        ..EnvironmentSchemaOpt::empty()
    };
    let environment_schema_opt = environment_schema_base_service
        .query_environment_schema_one(&params)
        .await?;
    let environment_schema = environment_schema_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("该环境规格不存在！"))
    })?;
    //查询已有的资源
    let environment_schema_resource_list = environment_schema_resource_base_service
        .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(id.into()),
            ..EnvironmentSchemaResourceOpt::empty()
        })
        .await?;
    return Ok(EnvironmentSchema {
        id: id.into(),
        name: environment_schema.name,
        resource_list: environment_schema_resource_list
            .into_iter()
            .map(|schema_resource| SchemaResource {
                id: schema_resource.id.into(),
                name: schema_resource.name,
                extension_id: schema_resource.extension_id,
                extension_name: schema_resource.extension_name,
            })
            .collect(),
    });
}
