use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;

pub const CONTINUE_JOB_API: &str = "/api/job/continueJob";

#[derive(Serialize, Deserialize, Debug)]
pub struct ContinueJobReq {
    pub record_id: Id,
    pub step_record_id: Id,
    pub success: bool,
}

pub type ContinueJobResp = ();
pub struct ContinueJobApi;
impl Api for ContinueJobApi {
    type Input = ContinueJobReq;
    type Output = ContinueJobResp;
    fn namespace() -> LightString {
        return LightString::from_static(CONTINUE_JOB_API);
    }
}
