use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;

pub const UPDATE_ENVIRONMENT_API: &str = "/api/environment/updateEnvironment";

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentResource {
    pub id: Option<Id>,
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
pub struct UpdateEnvironmentReq {
    pub id: Id,
    pub name: String,
    pub schema_resource_list: Vec<EnvironmentSchemaResource>,
}

pub type UpdateEnvironmentResp = ();

pub struct UpdateEnvironmentApi;
impl Api for UpdateEnvironmentApi {
    type Input = UpdateEnvironmentReq;
    type Output = UpdateEnvironmentResp;
    fn namespace() -> LightString {
        return LightString::from_static(UPDATE_ENVIRONMENT_API);
    }
}
