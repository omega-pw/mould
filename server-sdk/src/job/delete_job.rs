use tihu::Api;
use tihu::PrimaryKey;
use tihu::SharedString;

pub const DELETE_JOB_API: &str = "/api/job/deleteJob";

pub type DeleteJobReq = PrimaryKey;
pub type DeleteJobResp = ();
pub struct DeleteJobApi;
impl Api for DeleteJobApi {
    type Input = DeleteJobReq;
    type Output = DeleteJobResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(DELETE_JOB_API);
    }
}
