use crate::sdk;
use chrono::Utc;
use sdk::system::get_current_time::GetCurrentTimeResp;
use tihu_native::ErrNo;

pub async fn get_current_time() -> Result<GetCurrentTimeResp, ErrNo> {
    return Ok(GetCurrentTimeResp {
        current_time: Utc::now(),
    });
}
