use tihu::Api;
use tihu::LightString;
use tihu::PrimaryKey;

pub const DELETE_JOB_API: &str = "/api/job/deleteJob";

pub type DeleteJobReq = PrimaryKey;
pub type DeleteJobResp = ();
pub struct DeleteJobApi;
impl Api for DeleteJobApi {
    type Input = DeleteJobReq;
    type Output = DeleteJobResp;
    fn namespace() -> LightString {
        return LightString::from_static(DELETE_JOB_API);
    }
}
