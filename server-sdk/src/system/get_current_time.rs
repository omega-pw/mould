use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::Api;
use tihu::SharedString;

pub const GET_CURRENT_TIME_API: &str = "/api/system/getCurrentTime";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCurrentTimeReq {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetCurrentTimeResp {
    #[serde(with = "datetime_format")]
    pub current_time: DateTime<Utc>,
}

pub struct GetCurrentTimeApi;
impl Api for GetCurrentTimeApi {
    type Input = GetCurrentTimeReq;
    type Output = GetCurrentTimeResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(GET_CURRENT_TIME_API);
    }
}
