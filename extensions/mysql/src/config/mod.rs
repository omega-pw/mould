mod origin;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::File;
pub use origin::configuration_schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
    pub ssl: bool,
    pub root_cert: Option<File>,
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
        let dbname = origin_config.dbname.trim();
        if dbname.is_empty() {
            return Err(String::from("数据库不能为空"));
        }
        let user = origin_config.user.trim();
        if user.is_empty() {
            return Err(String::from("用户名不能为空"));
        }
        if origin_config.password.is_empty() {
            return Err(String::from("密码不能为空"));
        }
        return Ok(Config {
            host: host.to_string(),
            port: port,
            dbname: dbname.to_string(),
            user: user.to_string(),
            password: origin_config.password,
            ssl: origin_config.ssl,
            root_cert: if origin_config.ssl {
                origin_config.root_cert
            } else {
                None
            },
        });
    }
}
