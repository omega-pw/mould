mod origin;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
pub use origin::configuration_schema;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_addr: String,
    pub namespace: String,
    pub app_name: Option<String>,
    pub auth: Option<(String, String)>,
}

impl Config {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let origin_config =
            serde_json::from_value::<origin::Config>(value).map_err(|_err| -> String {
                return String::from("配置格式不正确");
            })?;
        let server_addr = origin_config.server_addr.trim();
        if server_addr.is_empty() {
            return Err(String::from("服务器地址不能为空"));
        }
        let namespace = origin_config.namespace.trim();
        let app_name = origin_config.app_name.trim();
        let username = origin_config.username.trim();
        let app_name = if app_name.is_empty() {
            None
        } else {
            Some(app_name.to_string())
        };
        let username_empty = username.is_empty();
        let password_empty = origin_config.password.is_empty();
        let auth = match (username_empty, password_empty) {
            (false, true) => {
                return Err(String::from("密码不能为空"));
            }
            (true, false) => {
                return Err(String::from("用户名不能为空"));
            }
            (false, false) => Some((username.to_string(), origin_config.password)),
            (true, true) => None,
        };
        return Ok(Config {
            server_addr: server_addr.to_string(),
            namespace: namespace.to_string(),
            app_name: app_name,
            auth: auth,
        });
    }
}
