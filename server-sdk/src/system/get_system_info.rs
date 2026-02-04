use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::Api;
use tihu::SharedString;

pub const GET_SYSTEM_INFO_API: &str = "/api/system/getSystemInfo";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSystemInfoReq {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetSystemInfoResp {
    pub version: String,
    #[serde(with = "datetime_format")]
    pub current_time: DateTime<Utc>,
    pub turnstile_site_key: Option<String>,
}

pub struct GetSystemInfoApi;
impl Api for GetSystemInfoApi {
    type Input = GetSystemInfoReq;
    type Output = GetSystemInfoResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(GET_SYSTEM_INFO_API);
    }
}
