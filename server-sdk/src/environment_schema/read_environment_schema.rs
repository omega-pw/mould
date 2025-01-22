use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;

pub const READ_ENVIRONMENT_SCHEMA_API: &str = "/api/environmentSchema/readEnvironmentSchema";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SchemaResource {
    pub id: Id,
    pub name: String,
    pub extension_id: String,
    pub extension_name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EnvironmentSchema {
    pub id: Id,
    pub name: String,
    pub resource_list: Vec<SchemaResource>,
}

pub type ReadEnvironmentSchemaReq = PrimaryKey;
pub type ReadEnvironmentSchemaResp = EnvironmentSchema;
pub struct ReadEnvironmentSchemaApi;
impl Api for ReadEnvironmentSchemaApi {
    type Input = ReadEnvironmentSchemaReq;
    type Output = ReadEnvironmentSchemaResp;
    fn namespace() -> LightString {
        return LightString::from_static(READ_ENVIRONMENT_SCHEMA_API);
    }
}
