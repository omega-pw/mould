pub mod parameter;
use super::await_task;
use super::get_session;
use super::upload_file;
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
) -> Result<(), String> {
    let append_log = append_log.clone();
    let result = await_task(&context.clone(), move || {
        try_handle(configuration, parameter, &append_log)
    })
    .await?;
    return result;
}

fn try_handle(
    configuration: Value,
    parameter: Value,
    append_log: &AppendLog,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    append_log(LogLevel::Info, String::from("正在连接服务器"));
    let (session, config) = get_session(configuration)?;
    append_log(LogLevel::Info, String::from("连接服务器成功"));
    if config.workspace.is_empty() {
        return Err(String::from("该操作工作目录不能为空"));
    }
    let workspace = config
        .workspace
        .strip_suffix("/")
        .unwrap_or(&config.workspace);
    let file_path = parameter
        .file_path
        .strip_prefix("/")
        .unwrap_or(&parameter.file_path);
    let remote_path = format!("{}/{}", workspace, file_path);
    append_log(LogLevel::Info, String::from("正在上传配置文件"));
    upload_file(&session, &remote_path, parameter.content.as_bytes(), 0o644)?;
    append_log(LogLevel::Info, String::from("修改成功!"));
    return Ok(());
}
