pub mod parameter;
use super::await_future;
use super::get_client;
use crate::config::Config;
use json5;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::api::{Api, PostParams};
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    context: &Context,
    append_log: &AppendLog,
    resource_index: u32,
) -> Result<(), String> {
    let configuration = Config::try_form_value(configuration)?;
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    let context = context.clone();
    let append_log = append_log.clone();
    let result = await_future(try_handle(
        configuration,
        parameter,
        context,
        append_log,
        resource_index,
    ))
    .await?;
    return result;
}

async fn try_handle(
    configuration: Config,
    parameter: Parameter,
    context: Context,
    append_log: AppendLog,
    resource_index: u32,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在构造kubernetes客户端"));
    let client = get_client(&configuration).await?;
    let config_maps: Api<ConfigMap> = Api::namespaced(client, &configuration.namespace);
    append_log(LogLevel::Info, String::from("正在获取ConfigMap"));
    let mut config_map = config_maps
        .get(&parameter.config_map_name)
        .await
        .map_err(|err| format!("获取ConfigMap失败: {err}"))?;
    let mut found = false;
    if let Some(data) = config_map.data.as_mut() {
        if let Some(content) = data.get_mut(&parameter.key) {
            append_log(LogLevel::Info, String::from("正在解析配置"));
            let root_value: Value =
                json5::from_str(content).map_err(|err| format!("解析json配置文件失败: {err}"))?;
            append_log(LogLevel::Info, String::from("正在修改配置"));
            let new_root_value = context.modify_json_custom(
                root_value,
                &parameter.json_path,
                &parameter.replace_function,
                resource_index,
            )?;
            append_log(LogLevel::Info, String::from("正在生成新配置"));
            let new_root_value = serde_json::to_string_pretty(&new_root_value)
                .map_err(|err| format!("序列化json配置文件失败: {err}"))?;
            *content = new_root_value;
            found = true;
        }
    }
    if !found {
        if let Some(binary_data) = config_map.binary_data.as_mut() {
            if let Some(content) = binary_data.get_mut(&parameter.key) {
                append_log(LogLevel::Info, String::from("正在解析配置"));
                let root_value = String::from_utf8(content.0.clone())
                    .map_err(|err| format!("原始配置不是UTF-8格式字符串: {err}"))?;
                let root_value: Value = json5::from_str(&root_value)
                    .map_err(|err| format!("解析json配置文件失败: {err}"))?;
                append_log(LogLevel::Info, String::from("正在修改配置"));
                let new_root_value = context.modify_json_custom(
                    root_value,
                    &parameter.json_path,
                    &parameter.replace_function,
                    resource_index,
                )?;
                append_log(LogLevel::Info, String::from("正在生成新配置"));
                let new_root_value = serde_json::to_vec_pretty(&new_root_value)
                    .map_err(|err| format!("序列化json配置文件失败: {err}"))?;
                content.0 = new_root_value;
                found = true;
            }
        }
    }
    if found {
        append_log(LogLevel::Info, String::from("正在提交新配置"));
        config_maps
            .replace(
                &parameter.config_map_name,
                &PostParams::default(),
                &config_map,
            )
            .await
            .map_err(|err| format!("更新ConfigMap失败: {err}"))?;
        append_log(LogLevel::Info, String::from("修改ConfigMap成功!"));
    } else {
        append_log(LogLevel::Warn, String::from("没有找到key对应的数据"));
    }
    return Ok(());
}
