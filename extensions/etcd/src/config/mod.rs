mod origin;
use etcd_rs::Endpoint;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
pub use origin::configuration_schema;

#[derive(Debug, Clone)]
pub struct Config {
    pub endpoints: Vec<Endpoint>,
    pub auth: Option<(String, String)>,
}

impl Config {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let origin_config =
            serde_json::from_value::<origin::Config>(value).map_err(|_err| -> String {
                return String::from("配置格式不正确");
            })?;
        let endpoints = origin_config
            .endpoints
            .split(',')
            .map(|endpoint| endpoint.trim())
            .filter(|endpoint| !endpoint.is_empty())
            .map(From::from)
            .collect::<Vec<Endpoint>>();
        if endpoints.is_empty() {
            return Err(String::from("endpoints配置为空!"));
        }
        let user = origin_config.user.trim();
        let user_empty = user.is_empty();
        let password_empty = origin_config.password.is_empty();
        let auth = match (user_empty, password_empty) {
            (false, true) => {
                return Err(String::from("密码不能为空"));
            }
            (true, false) => {
                return Err(String::from("用户名不能为空"));
            }
            (false, false) => Some((user.to_string(), origin_config.password)),
            (true, true) => None,
        };
        return Ok(Config {
            endpoints: endpoints,
            auth: auth,
        });
    }
}
