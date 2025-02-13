mod origin;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
pub use origin::configuration_schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub api_token: String,
    pub account_id: String,
    pub project_name: String,
}

impl Config {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let origin_config =
            serde_json::from_value::<origin::Config>(value).map_err(|_err| -> String {
                return String::from("配置格式不正确");
            })?;
        let api_token = origin_config.api_token.trim();
        if api_token.is_empty() {
            return Err(String::from("Api Token不能为空"));
        }
        let account_id = origin_config.account_id.trim();
        if account_id.is_empty() {
            return Err(String::from("Account Id不能为空"));
        }
        let project_name = origin_config.project_name.trim();
        if project_name.is_empty() {
            return Err(String::from("Project Name不能为空"));
        }
        return Ok(Config {
            api_token: api_token.to_string(),
            account_id: account_id.to_string(),
            project_name: project_name.to_string(),
        });
    }
}
