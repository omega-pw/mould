use super::NONCE_PREFIX;
use crate::log;
use crate::native_common;
use crate::prehandle::auth::Guest;
use crate::sdk;
use crate::Context;
use native_common::cache::AsyncCache;
use native_common::cache::EliminateType;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::get_nonce::GetNonceResp;
use std::sync::Arc;
use tihu::base62;
use tihu_native::ErrNo;
use uuid::Uuid;

pub async fn get_nonce(
    context: Arc<Context>,
    _guest: Guest,
    _get_nonce_req: GetNonceReq,
) -> Result<GetNonceResp, ErrNo> {
    let cache_mgr = context.get_cache_mgr().await?;
    let nonce = base62::encode(&Uuid::new_v4().as_u128().to_be_bytes());
    let nonce_val: Vec<u8> = vec![0x00]; //具体的值是什么无所谓
    cache_mgr
        .set(
            &(String::from(NONCE_PREFIX) + &nonce).into_bytes(),
            &nonce_val,
            EliminateType::Expire(30 * 1000), //30秒过期
        )
        .await
        .map_err(|err| {
            log::error!("缓存nonce数据失败: {:?}", err);
            err
        })?;
    return Ok(nonce);
}
