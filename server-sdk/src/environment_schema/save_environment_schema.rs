use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::PrimaryKey;
use tihu::SharedString;

pub const SAVE_ENVIRONMENT_SCHEMA_API: &str = "/api/environmentSchema/saveEnvironmentSchema";

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaResource {
    pub id: Option<Id>,
    pub name: String,
    pub extension_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveEnvironmentSchemaReq {
    pub id: Option<Id>,
    pub name: String,
    pub resource_list: Vec<SchemaResource>,
}

pub type SaveEnvironmentSchemaResp = PrimaryKey;
pub struct SaveEnvironmentSchemaApi;
impl Api for SaveEnvironmentSchemaApi {
    type Input = SaveEnvironmentSchemaReq;
    type Output = SaveEnvironmentSchemaResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(SAVE_ENVIRONMENT_SCHEMA_API);
    }
}
