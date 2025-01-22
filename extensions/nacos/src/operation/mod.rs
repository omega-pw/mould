pub mod modify_properties;
pub mod put;
pub mod test;
use crate::config::Config;
use mould_extension_sdk::serde_json::Value;
use nacos_sdk::api::config::ConfigService;
use nacos_sdk::api::config::ConfigServiceBuilder;
use nacos_sdk::api::props::ClientProps;

pub async fn get_client(configuration: Value) -> Result<impl ConfigService, String> {
    let configuration = Config::try_form_value(configuration)?;
    let server_addr_prefix = format!("服务器地址：{:?}, ", configuration.server_addr);
    let mut props = ClientProps::new()
        .server_addr(configuration.server_addr)
        .namespace(configuration.namespace);
    if let Some(app_name) = configuration.app_name {
        props = props.app_name(app_name);
    }
    if let Some((username, password)) = configuration.auth {
        props = props.auth_username(username).auth_password(password);
    }
    let config_service = ConfigServiceBuilder::new(props)
        .build()
        .map_err(|err| format!("{server_addr_prefix}连接nacos服务器失败: {err}"))?;
    return Ok(config_service);
}
