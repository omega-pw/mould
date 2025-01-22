use json5;
use oauth2::Scope;
use object_storage_lib::Oss;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::net::IpAddr;
use tihu::LightString;

#[derive(Serialize, Deserialize, Debug)]
pub struct CacheServer {
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
    pub max_size: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SslCfg {
    pub root_cert: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSource {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
    pub max_size: Option<usize>,
    pub ssl: Option<SslCfg>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAccount {
    pub mail_host: String,
    pub mail_port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Oauth2Server {
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(default)]
    pub pkce: bool,
    pub scopes: Option<Vec<Scope>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenidServer {
    pub name: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailTemplate {
    pub register_captcha: String,
    pub reset_password_captcha: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub host: Option<IpAddr>,
    pub port: Option<u16>,
    pub log_cfg_path: Option<String>,
    pub extension_dir: String,
    pub job_log_dir: String,
    // pub worker_id: u16,
    pub sign_secret: String,
    // pub session_timeout: u8,
    pub rsa_pub_key: String,
    pub rsa_pri_key: String,
    pub server_random_value: String,
    pub cache_server: CacheServer,
    pub data_source: DataSource,
    pub oss: Oss,
    pub public_path: String,
    #[serde(default)]
    pub oauth2_servers: HashMap<String, Oauth2Server>,
    #[serde(default)]
    pub openid_servers: HashMap<String, OpenidServer>,
    pub email_account: EmailAccount,
    pub email_template: EmailTemplate,
}

impl Config {
    pub fn try_load_from_file(file_path: &str) -> Result<Self, LightString> {
        let content = read_to_string(&file_path).map_err(|err| {
            log::error!("read file to string error: {}", err);
            return LightString::from_static("read file to string error");
        })?;
        let config: Config = json5::from_str(&content).map_err(|err| {
            log::error!("Invalid configuration file, {:?}", err);
            return LightString::from(format!("Invalid configuration file, {:?}", err));
        })?;
        return Ok(config);
    }
}
