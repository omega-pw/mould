pub mod parameter;
use super::get_client;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::LogLevel;
use nacos_sdk::api::config::ConfigService;
use parameter::Parameter;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    append_log: &AppendLog,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    append_log(LogLevel::Info, String::from("正在连接nacos服务器"));
    let client = get_client(configuration).await?;
    append_log(LogLevel::Info, String::from("连接nacos服务器成功"));
    let key_prefix = format!(
        "配置ID: {}, 配置组: {}, ",
        parameter.data_id, parameter.group
    );
    let content_type = if parameter.content_type.is_empty() {
        None
    } else {
        Some(parameter.content_type.clone())
    };
    append_log(LogLevel::Info, String::from("正在写入配置"));
    let ok = client
        .publish_config(
            parameter.data_id.clone(),
            parameter.group.clone(),
            parameter.content.clone(),
            content_type,
        )
        .await
        .map_err(|err| format!("{key_prefix}设置失败: {}, 内容: {}", err, parameter.content))?;
    if !ok {
        return Err(format!("{key_prefix}设置失败!"));
    }
    append_log(LogLevel::Info, String::from("设置成功!"));
    return Ok(());
}
