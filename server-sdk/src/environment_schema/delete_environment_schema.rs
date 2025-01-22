use tihu::Api;
use tihu::LightString;
use tihu::PrimaryKey;

pub const DELETE_ENVIRONMENT_SCHEMA_API: &str = "/api/environmentSchema/deleteEnvironmentSchema";

pub type DeleteEnvironmentSchemaReq = PrimaryKey;
pub type DeleteEnvironmentSchemaResp = ();
pub struct DeleteEnvironmentSchemaApi;
impl Api for DeleteEnvironmentSchemaApi {
    type Input = DeleteEnvironmentSchemaReq;
    type Output = DeleteEnvironmentSchemaResp;
    fn namespace() -> LightString {
        return LightString::from_static(DELETE_ENVIRONMENT_SCHEMA_API);
    }
}
