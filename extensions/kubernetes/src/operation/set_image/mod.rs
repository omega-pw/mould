pub mod parameter;
use super::await_future;
use super::get_client;
use crate::config::Config;
use k8s_openapi::api::apps::v1::Deployment;
use kube::api::{Api, PostParams};
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
    append_log(LogLevel::Info, String::from("正在获取工作负载"));
    let mut deployment = deployments
        .get(&parameter.deployment_name)
        .await
        .map_err(|err| format!("获取工作负载失败: {err}"))?;
    let mut found = false;
    if let Some(spec) = deployment.spec.as_mut() {
        if let Some(spec) = spec.template.spec.as_mut() {
            for container in &mut spec.containers {
                if parameter.container_name.is_none()
                    || parameter.container_name.as_deref() == Some(container.name.as_str())
                {
                    container.image.replace(parameter.image);
                    found = true;
                    break;
                }
            }
        }
    }
    if found {
        append_log(LogLevel::Info, String::from("正在修改镜像"));
        deployments
            .replace(
                &parameter.deployment_name,
                &PostParams::default(),
                &deployment,
            )
            .await
            .map_err(|err| format!("更新工作负载镜像失败: {err}"))?;
        append_log(LogLevel::Info, String::from("修改镜像成功!"));
    } else {
        append_log(LogLevel::Warn, String::from("没有找到工作负载"));
    }
    return Ok(());
}
