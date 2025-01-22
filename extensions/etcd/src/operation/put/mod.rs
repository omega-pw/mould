pub mod parameter;
use super::get_client;
use etcd_rs::KeyValueOp;
use etcd_rs::PutRequest;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    append_log: AppendLog,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    append_log(LogLevel::Info, String::from("正在连接etcd服务器"));
    let client = get_client(configuration).await?;
    append_log(LogLevel::Info, String::from("连接etcd服务器成功"));
    let req = PutRequest::new(parameter.key.clone(), parameter.value.clone());
    append_log(LogLevel::Info, String::from("正在写入配置"));
    client.put(req).await.map_err(|err| {
        format!(
            "设置失败: {}, key: {}, value: {}",
            err, parameter.key, parameter.value
        )
    })?;
    append_log(LogLevel::Info, String::from("设置成功!"));
    return Ok(());
}
