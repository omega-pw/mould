mod origin;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
pub use origin::configuration_schema;

#[derive(Debug, Clone)]
pub struct Config {
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
}

impl Config {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let origin_config =
            serde_json::from_value::<origin::Config>(value).map_err(|_err| -> String {
                return String::from("配置格式不正确");
            })?;
        let access_key = origin_config.access_key.trim();
        if access_key.is_empty() {
            return Err(String::from("access key不能为空"));
        }
        if origin_config.secret_key.is_empty() {
            return Err(String::from("secret key不能为空"));
        }
        let endpoint = origin_config.endpoint.trim();
        if endpoint.is_empty() {
            return Err(String::from("endpoint不能为空"));
        }
        let region = origin_config.region.trim();
        if region.is_empty() {
            return Err(String::from("region不能为空"));
        }
        let bucket = origin_config.bucket.trim();
        if bucket.is_empty() {
            return Err(String::from("bucket不能为空"));
        }
        return Ok(Config {
            access_key: access_key.to_string(),
            secret_key: origin_config.secret_key,
            endpoint: endpoint.to_string(),
            region: region.to_string(),
            bucket: bucket.to_string(),
        });
    }
}
