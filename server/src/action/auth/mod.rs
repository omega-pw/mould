pub mod change_password;
pub mod get_curr_user;
pub mod get_nonce;
pub mod get_openid_providers;
pub mod get_rsa_pub_key;
pub mod get_salt;
pub mod login;
pub mod login_by_oauth2_code;
pub mod login_by_openid_code;
pub mod logout;
pub mod register;
pub mod reset_password;
pub mod send_email_captcha;
use crate::get_context;
use crate::middleware::auth::SessionInfo;
use crate::middleware::auth::SESSION_PREFIX;
use crate::middleware::session::SessionId;
use crate::native_common;
use native_common::cache::AsyncCache;
use native_common::cache::EliminateType;
use native_common::utils::decrypt_by_base64;
use native_common::utils::decrypt_by_rsa_pri_key;
use rsa::RsaPrivateKey;
use tihu::LightString;
use tihu_native::ErrNo;

const NONCE_PREFIX: &'static str = "nonce-";

async fn check_nonce(nonce: &str) -> Result<bool, ErrNo> {
    let context = get_context()?;
    let cache_mgr = context.get_cache_mgr().await?;
    return cache_mgr
        .remove(&(String::from(NONCE_PREFIX) + &nonce).into_bytes())
        .await;
}

fn decrypt_base64_data_by_rsa_pri_key(
    content: &str,
    name: &str,
    nonce: &[u8],
    rsa_pri_key: &RsaPrivateKey,
) -> Result<Vec<u8>, LightString> {
    let content_bytes = decrypt_by_base64(content).map_err(|err| -> LightString {
        log::error!("base64解码{}失败: {:?}", name, err);
        let err_msg = format!("解码{}失败！", name);
        return err_msg.into();
    })?;
    let mut content =
        decrypt_by_rsa_pri_key(&content_bytes, rsa_pri_key).map_err(|err| -> LightString {
            log::error!("{}解密失败: {:?},密文：{}", name, err, content);
            let err_msg = format!("{}解密失败！", name);
            return err_msg.into();
        })?;
    //把nonce放在实际数据后边，方便字符串移除，如果放在前面，还要新生成一个字符串
    if !content.ends_with(nonce) {
        return Err(LightString::from_static("非法访问，数据签名校验失败！"));
    } else {
        let drop_len = nonce.len();
        content.truncate(content.len() - drop_len);
    }
    Ok(content)
}

async fn cache_session_info(
    session_id: SessionId,
    session_info: &SessionInfo,
) -> Result<(), ErrNo> {
    let context = get_context()?;
    let session_info = serde_json::to_vec(&session_info).map_err(ErrNo::SerializeError)?;
    let session_id = session_id.to_string();
    let cache_mgr = context.get_cache_mgr().await?;
    cache_mgr
        .set(
            &(String::from(SESSION_PREFIX) + &session_id).into_bytes(),
            &session_info,
            EliminateType::Expire(30 * 60 * 1000), //30分钟过期
        )
        .await
        .map_err(|err| {
            log::error!("缓存token数据失败: {:?}", err);
            err
        })?;
    return Ok(());
}
