pub mod parameter;
use super::await_future;
use super::get_client;
use crate::config::Config;
use k8s_openapi::api::apps::v1::Deployment;
use kube::api::Api;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    _context: &Context,
    append_log: &AppendLog,
) -> Result<(), String> {
    let configuration = Config::try_form_value(configuration)?;
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    let result = await_future(try_handle(configuration, parameter, append_log.clone())).await?;
    return result;
}

async fn try_handle(
    configuration: Config,
    parameter: Parameter,
    append_log: AppendLog,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在构造kubernetes客户端"));
    let client = get_client(&configuration).await?;
    let deployments: Api<Deployment> = Api::namespaced(client, &configuration.namespace);
    append_log(LogLevel::Info, String::from("正在重启工作负载"));
    deployments
        .restart(&parameter.deployment_name)
        .await
        .map_err(|err| format!("重启工作负载失败: {err}"))?;
    append_log(LogLevel::Info, String::from("重启工作负载完成!"));
    return Ok(());
}
