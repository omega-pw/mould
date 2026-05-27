use crate::context::Context;
use crate::sdk;
use crate::VERSION_INFO;
use chrono::Utc;
use sdk::system::get_system_info::GetSystemInfoResp;
use std::sync::Arc;
use tihu_native::ErrNo;

pub async fn get_system_info(_context: Arc<Context>) -> Result<GetSystemInfoResp, ErrNo> {
    return Ok(GetSystemInfoResp {
        version: VERSION_INFO.to_string(),
        current_time: Utc::now(),
    });
}
