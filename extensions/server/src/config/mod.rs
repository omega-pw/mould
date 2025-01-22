mod origin;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
pub use origin::configuration_schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
    pub workspace: String,
}

impl Config {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let origin_config =
            serde_json::from_value::<origin::Config>(value).map_err(|_err| -> String {
                return String::from("配置格式不正确");
            })?;
        let host = origin_config.host.trim();
        if host.is_empty() {
            return Err(String::from("主机不能为空"));
        }
        let port = u16::from_str_radix(&origin_config.port, 10)
            .map_err(|_err| String::from("端口不正确"))?;
        let user = origin_config.user.trim();
        if user.is_empty() {
            return Err(String::from("用户名不能为空"));
        }
        let private_key = origin_config.private_key.trim();
        if private_key.is_empty() && origin_config.password.is_empty() {
            return Err(String::from("密码不能为空"));
        }
        let public_key = origin_config.public_key.trim();
        let workspace = origin_config.workspace.trim();
        return Ok(Config {
            host: host.to_string(),
            port: port,
            user: user.to_string(),
            password: origin_config.password,
            public_key: public_key.to_string(),
            private_key: private_key.to_string(),
            passphrase: origin_config.passphrase,
            workspace: workspace.to_string(),
        });
    }
}
