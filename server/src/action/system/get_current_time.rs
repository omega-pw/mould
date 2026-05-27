use crate::middleware::auth::Guest;
use crate::sdk;
use chrono::Utc;
use sdk::system::get_current_time::GetCurrentTimeReq;
use sdk::system::get_current_time::GetCurrentTimeResp;
use tihu_native::ErrNo;

pub async fn get_current_time(
    _guest: Guest,
    _get_get_current_time_req: GetCurrentTimeReq,
) -> Result<GetCurrentTimeResp, ErrNo> {
    return Ok(GetCurrentTimeResp {
        current_time: Utc::now(),
    });
}
