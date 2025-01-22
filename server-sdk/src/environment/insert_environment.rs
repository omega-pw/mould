use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;

pub const INSERT_ENVIRONMENT_API: &str = "/api/environment/insertEnvironment";

/**
 * 环境资源
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentResource {
    pub name: String,
    pub extension_configuration: String, //扩展配置
}

/**
 * 环境规格资源
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentSchemaResource {
    pub id: Id,
    pub extension_id: String,
    pub resource_list: Vec<EnvironmentResource>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertEnvironmentReq {
    pub environment_schema_id: Id,
    pub name: String,
    pub schema_resource_list: Vec<EnvironmentSchemaResource>,
}

pub type InsertEnvironmentResp = PrimaryKey;
pub struct InsertEnvironmentApi;
impl Api for InsertEnvironmentApi {
    type Input = InsertEnvironmentReq;
    type Output = InsertEnvironmentResp;
    fn namespace() -> LightString {
        return LightString::from_static(INSERT_ENVIRONMENT_API);
    }
}
