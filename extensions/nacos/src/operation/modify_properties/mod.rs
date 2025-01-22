pub mod parameter;
use super::get_client;
use java_properties::PropertiesIter;
use java_properties::PropertiesWriter;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::LogLevel;
use nacos_sdk::api::config::ConfigService;
use parameter::Parameter;
use std::collections::HashMap;
use std::io::Cursor;

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
        "配置ID: {}, 配置组: {}, key: {}, ",
        parameter.data_id, parameter.group, parameter.key
    );
    append_log(LogLevel::Info, String::from("正在获取配置"));
    let resp = client
        .get_config(parameter.data_id.clone(), parameter.group.clone())
        .await
        .map_err(|err| format!("{key_prefix}获取properties配置失败: {err}"))?;
    let content = resp.content();
    let mut map = HashMap::new();
    append_log(LogLevel::Info, String::from("正在解析配置"));
    PropertiesIter::new(Cursor::new(content))
        .read_into(|key, value| {
            map.insert(key, value);
        })
        .map_err(|err| format!("{key_prefix}解析properties配置失败: {err}, 内容：{content}"))?;
    map.insert(parameter.key, parameter.value);
    append_log(LogLevel::Info, String::from("正在重新生成配置"));
    let mut new_content: Vec<u8> = Vec::new();
    let mut writer = PropertiesWriter::new(Cursor::new(&mut new_content));
    for (key, value) in &map {
        writer
            .write(key, value)
            .map_err(|err| format!("{key_prefix}写入properties配置失败: {err}"))?;
    }
    writer
        .finish()
        .map_err(|err| format!("{key_prefix}写入properties配置失败: {err}"))?;
    let new_content = String::from_utf8(new_content)
        .map_err(|err| format!("{key_prefix}properties配置转成utf8字符串失败: {err}"))?;
    append_log(LogLevel::Info, String::from("正在写入配置"));
    let ok = client
        .publish_config(
            parameter.data_id.clone(),
            parameter.group.clone(),
            new_content.clone(),
            Some(String::from("text/plain")),
        )
        .await
        .map_err(|err| format!("{key_prefix}设置properties配置失败: {err}, 内容: {new_content}"))?;
    if !ok {
        return Err(format!("{key_prefix}设置properties配置失败!"));
    }
    append_log(LogLevel::Info, String::from("修改成功!"));
    return Ok(());
}
