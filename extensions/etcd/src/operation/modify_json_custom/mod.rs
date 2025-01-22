pub mod parameter;
use super::get_client;
use etcd_rs::KeyValueOp;
use etcd_rs::PutRequest;
use json5;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    context: Context,
    append_log: AppendLog,
    resource_index: u32,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    append_log(LogLevel::Info, String::from("正在连接etcd服务器"));
    let client = get_client(configuration).await?;
    append_log(LogLevel::Info, String::from("连接etcd服务器成功"));
    append_log(LogLevel::Info, String::from("正在获取配置"));
    let get_resp = client
        .get(parameter.key.as_str())
        .await
        .map_err(|err| format!("获取json配置失败: {err}"))?;
    let key_prefix = format!("key：{}, ", parameter.key);
    append_log(LogLevel::Info, String::from("正在修改配置"));
    for kv in get_resp.kvs {
        let key = kv.key;
        let value = kv.value;
        let value = String::from_utf8(value)
            .map_err(|err| format!("{key_prefix}原始配置不是UTF-8格式字符串: {err}"))?;
        let root_value: Value = json5::from_str(&value)
            .map_err(|err| format!("{key_prefix}解析json配置失败: {err}"))?;
        let new_root_value = context.modify_json_custom(
            root_value,
            &parameter.json_path,
            &parameter.replace_function,
            resource_index,
        )?;
        let new_root_value = serde_json::to_vec_pretty(&new_root_value)
            .map_err(|err| format!("{key_prefix}序列化json配置失败: {err}"))?;
        let req = PutRequest::new(key, new_root_value);
        client
            .put(req)
            .await
            .map_err(|err| format!("{key_prefix}写入json配置失败: {err}"))?;
    }
    append_log(LogLevel::Info, String::from("修改成功!"));
    return Ok(());
}
