use tihu::Api;
use tihu::LightString;
use tihu::PrimaryKey;

pub const DELETE_ENVIRONMENT_API: &str = "/api/environment/deleteEnvironment";

pub type DeleteEnvironmentReq = PrimaryKey;
pub type DeleteEnvironmentResp = ();
pub struct DeleteEnvironmentApi;
impl Api for DeleteEnvironmentApi {
    type Input = DeleteEnvironmentReq;
    type Output = DeleteEnvironmentResp;
    fn namespace() -> LightString {
        return LightString::from_static(DELETE_ENVIRONMENT_API);
    }
}
