use crate::sdk;
use crate::VERSION_INFO;
use chrono::Utc;
use sdk::system::get_system_info::GetSystemInfoResp;
use tihu_native::ErrNo;

pub async fn get_system_info() -> Result<GetSystemInfoResp, ErrNo> {
    return Ok(GetSystemInfoResp {
        version: VERSION_INFO.to_string(),
        current_time: Utc::now(),
    });
}
