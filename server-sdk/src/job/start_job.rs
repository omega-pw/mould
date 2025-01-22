use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;

pub const START_JOB_API: &str = "/api/job/startJob";

#[derive(Serialize, Deserialize, Debug)]
pub struct StartJobReq {
    pub job_id: Id,
    pub environment_id: Id,
}

pub type StartJobResp = PrimaryKey;
pub struct StartJobApi;
impl Api for StartJobApi {
    type Input = StartJobReq;
    type Output = StartJobResp;
    fn namespace() -> LightString {
        return LightString::from_static(START_JOB_API);
    }
}
