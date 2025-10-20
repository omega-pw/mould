use tihu::Api;
use tihu::PrimaryKey;
use tihu::SharedString;

pub const DELETE_ENVIRONMENT_SCHEMA_API: &str = "/api/environmentSchema/deleteEnvironmentSchema";

pub type DeleteEnvironmentSchemaReq = PrimaryKey;
pub type DeleteEnvironmentSchemaResp = ();
pub struct DeleteEnvironmentSchemaApi;
impl Api for DeleteEnvironmentSchemaApi {
    type Input = DeleteEnvironmentSchemaReq;
    type Output = DeleteEnvironmentSchemaResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(DELETE_ENVIRONMENT_SCHEMA_API);
    }
}
