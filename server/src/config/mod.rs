mod args;
mod origin;

use crate::native_common;
pub use args::Arguments;
use native_common::utils::new_rsa_pri_key;
use native_common::utils::new_rsa_pub_key;
use object_storage_lib::Oss;
pub use origin::CacheServer;
pub use origin::DataSource;
pub use origin::EmailAccount;
pub use origin::EmailTemplate;
pub use origin::Oauth2Server;
pub use origin::OpenidServer;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tihu::base62;
use tihu::LightString;

#[derive(Debug)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub log_cfg_path: Option<LightString>,
    pub extension_dir: String,
    pub job_log_dir: String,
    // pub worker_id: u16,
    pub sign_secret: Arc<Vec<u8>>,
    // pub session_timeout: u8,
    pub rsa_pub_key: RsaPublicKey,
    pub rsa_pri_key: RsaPrivateKey,
    pub rsa_pub_key_content: LightString,
    pub server_random_value: [u8; 32],
    pub cache_server: CacheServer,
    pub data_source: DataSource,
    pub oss: Oss,
    pub public_path: String,
    pub oauth2_servers: HashMap<String, Oauth2Server>,
    pub openid_servers: HashMap<String, OpenidServer>,
    pub email_account: Arc<EmailAccount>,
    pub email_template: EmailTemplate,
}

impl Config {
    pub fn try_load_from_file(file_path: &str) -> Result<Self, LightString> {
        let config = origin::Config::try_load_from_file(file_path)?;
        let mut job_log_dir = config.job_log_dir;
        if job_log_dir.ends_with(&['/', '\\']) {
            job_log_dir.pop();
        }
        let host = config.host.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let sign_secret = Arc::new(config.sign_secret.as_bytes().to_vec());
        let port = config.port.unwrap_or(80);
        let rsa_pub_key_content = read_to_string(&config.rsa_pub_key).map_err(|err| {
            log::error!("read server public key failed: {}", err);
            return LightString::from_static("read server public key failed");
        })?;
        let rsa_pub_key = new_rsa_pub_key(&rsa_pub_key_content)?;
        let rsa_pri_key = read_to_string(&config.rsa_pri_key).map_err(|err| {
            log::error!("read server private key failed: {}", err);
            return LightString::from_static("read server private key failed");
        })?;
        let rsa_pri_key = new_rsa_pri_key(&rsa_pri_key)?;
        let server_random_value = try_decode_bytes(&config.server_random_value, "服务端随机数")?;
        let register_captcha_template = read_to_string(&config.email_template.register_captcha)
            .map_err(|err| {
                log::error!("读取注册邮件模板文件失败: {}", err);
                return LightString::from_static(
                    "Failed to read the registration email template file",
                );
            })?;
        let reset_password_captcha_template =
            read_to_string(&config.email_template.reset_password_captcha).map_err(|err| {
                log::error!("读取重置密码邮件模板文件失败: {}", err);
                return LightString::from_static("Failed to read the reset email template file");
            })?;
        let email_template = EmailTemplate {
            register_captcha: register_captcha_template,
            reset_password_captcha: reset_password_captcha_template,
        };
        return Ok(Config {
            host: host,
            port: port,
            log_cfg_path: config.log_cfg_path.map(From::from),
            extension_dir: config.extension_dir,
            job_log_dir: job_log_dir,
            // worker_id: config.worker_id,
            sign_secret: sign_secret,
            // session_timeout: config.session_timeout,
            rsa_pub_key: rsa_pub_key,
            rsa_pri_key: rsa_pri_key,
            rsa_pub_key_content: rsa_pub_key_content.into(),
            server_random_value: server_random_value,
            cache_server: config.cache_server,
            data_source: config.data_source,
            oss: config.oss,
            public_path: config.public_path,
            oauth2_servers: config.oauth2_servers,
            openid_servers: config.openid_servers,
            email_account: Arc::new(config.email_account),
            email_template: email_template,
        });
    }
}

fn try_decode_bytes<const N: usize>(value: &str, name: &str) -> Result<[u8; N], LightString> {
    let bytes = base62::decode(value).map_err(|err| {
        log::error!("{}不是base62编码: {:?}", name, err);
        LightString::from(format!("{}不是base62编码", name))
    })?;
    if N != bytes.len() {
        log::error!("{}位数不正确!", name);
        return Err(LightString::from(format!("{}位数不正确!", name)));
    }
    let mut data = [0u8; N];
    data.copy_from_slice(&bytes);
    return Ok(data);
}
