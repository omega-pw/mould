use crate::get_context;
use crate::middleware::auth::User;
use crate::model::environment::EnvironmentOpt;
use crate::model::environment_resource::EnvironmentResourceOpt;
use crate::model::environment_schema_resource::EnvironmentSchemaResourceOpt;
use crate::native_common::utils::list::group_sub_list;
use crate::sdk;
use crate::service::base::EnvironmentBaseService;
use crate::service::base::EnvironmentResourceBaseService;
use crate::service::base::EnvironmentSchemaResourceBaseService;
use sdk::environment::read_environment::Environment;
use sdk::environment::read_environment::EnvironmentResource;
use sdk::environment::read_environment::EnvironmentSchemaResource;
use sdk::environment::read_environment::ReadEnvironmentReq;
use tihu::Id;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn read_environment(
    org_id: Id,
    _user: User,
    read_environment_req: ReadEnvironmentReq,
) -> Result<Environment, ErrNo> {
    let ReadEnvironmentReq { id } = read_environment_req;
    let environment_id = id;
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let environment_base_service = EnvironmentBaseService::new(&transaction);
    let environment_resource_base_service = EnvironmentResourceBaseService::new(&transaction);
    let environment_schema_resource_base_service =
        EnvironmentSchemaResourceBaseService::new(&transaction);
    let params = EnvironmentOpt {
        org_id: Some(org_id),
        id: Some(environment_id),
        ..EnvironmentOpt::empty()
    };
    let environment_opt = environment_base_service
        .query_environment_one(&params)
        .await?;
    let environment = environment_opt.ok_or_else(|| -> ErrNo {
        ErrNo::CommonError(LightString::from_static("该环境不存在！"))
    })?;
    //查询环境需要哪些资源
    let environment_schema_resource_list = environment_schema_resource_base_service
        .query_environment_schema_resource_batch(&EnvironmentSchemaResourceOpt {
            org_id: Some(org_id),
            environment_schema_id: Some(environment.environment_schema_id),
            ..EnvironmentSchemaResourceOpt::empty()
        })
        .await?;
    let resource_list = environment_resource_base_service
        .query_environment_resource_batch(&EnvironmentResourceOpt {
            org_id: Some(org_id),
            environment_id: Some(environment_id.into()),
            ..EnvironmentResourceOpt::empty()
        })
        .await?;
    let schema_resource_list = group_sub_list(
        environment_schema_resource_list,
        resource_list,
        |environment_schema_resource, resource| {
            environment_schema_resource.id == resource.schema_resource_id
        },
    );
    return Ok(Environment {
        id: id.into(),
        environment_schema_id: environment.environment_schema_id.into(),
        name: environment.name,
        schema_resource_list: schema_resource_list
            .into_iter()
            .map(
                |(schema_resource, resource_list)| EnvironmentSchemaResource {
                    id: schema_resource.id.into(),
                    name: schema_resource.name,
                    extension_id: schema_resource.extension_id,
                    extension_name: schema_resource.extension_name,
                    resource_list: resource_list
                        .into_iter()
                        .map(|resource| EnvironmentResource {
                            id: resource.id.into(),
                            name: resource.name,
                            extension_configuration: resource.extension_configuration,
                        })
                        .collect(),
                },
            )
            .collect(),
    });
}
