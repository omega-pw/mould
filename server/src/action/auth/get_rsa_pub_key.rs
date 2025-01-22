use crate::get_context;
use crate::middleware::auth::Guest;
use crate::sdk;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyReq;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyResp;
use tihu_native::ErrNo;

pub async fn get_rsa_pub_key(
    _guest: Guest,
    _get_rsa_pub_key_req: GetRsaPubKeyReq,
) -> Result<GetRsaPubKeyResp, ErrNo> {
    let context = get_context()?;
    let get_rsa_pub_key_content = context.get_rsa_pub_key_content().await.map_err(|err| {
        log::error!("获取公钥失败: {}", err);
        err
    })?;
    return Ok(get_rsa_pub_key_content.to_string());
}
