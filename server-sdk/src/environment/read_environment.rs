use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;

pub const READ_ENVIRONMENT_API: &str = "/api/environment/readEnvironment";

/**
 * 环境资源
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnvironmentResource {
    pub id: Id,                          //id
    pub name: String,                    //资源名称
    pub extension_configuration: String, //扩展配置
}

/**
 * 环境规格资源
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnvironmentSchemaResource {
    pub id: Id,
    pub name: String,
    pub extension_id: String,
    pub extension_name: String,
    pub resource_list: Vec<EnvironmentResource>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Environment {
    pub id: Id,
    pub environment_schema_id: Id,
    pub name: String,
    pub schema_resource_list: Vec<EnvironmentSchemaResource>,
}

pub type ReadEnvironmentReq = PrimaryKey;
pub type ReadEnvironmentResp = Environment;
pub struct ReadEnvironmentApi;
impl Api for ReadEnvironmentApi {
    type Input = ReadEnvironmentReq;
    type Output = ReadEnvironmentResp;
    fn namespace() -> LightString {
        return LightString::from_static(READ_ENVIRONMENT_API);
    }
}
