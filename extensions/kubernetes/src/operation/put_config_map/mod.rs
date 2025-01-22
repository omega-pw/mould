pub mod parameter;
use super::await_future;
use super::get_client;
use crate::config::Config;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::ByteString;
use kube::api::{Api, PostParams};
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;
use std::collections::BTreeMap;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    _context: &Context,
    append_log: &AppendLog,
) -> Result<(), String> {
    let configuration = Config::try_form_value(configuration)?;
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    let append_log = append_log.clone();
    let result = await_future(try_handle(configuration, parameter, append_log)).await?;
    return result;
}

async fn try_handle(
    configuration: Config,
    parameter: Parameter,
    append_log: AppendLog,
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
            *content = parameter.content.clone();
            found = true;
        }
    }
    if !found {
        if let Some(binary_data) = config_map.binary_data.as_mut() {
            if let Some(content) = binary_data.get_mut(&parameter.key) {
                content.0 = parameter.content.clone().into_bytes();
                found = true;
            }
        }
    }
    if !found {
        if let Some(data) = config_map.data.as_mut() {
            data.insert(parameter.key.clone(), parameter.content.clone());
            found = true;
        }
        if !found {
            if let Some(binary_data) = config_map.binary_data.as_mut() {
                binary_data.insert(
                    parameter.key.clone(),
                    ByteString(parameter.content.clone().into_bytes()),
                );
                found = true;
            }
        }
        if !found {
            let mut data: BTreeMap<String, String> = BTreeMap::new();
            data.insert(parameter.key, parameter.content);
            config_map.data.replace(data);
        }
    }
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
    return Ok(());
}
