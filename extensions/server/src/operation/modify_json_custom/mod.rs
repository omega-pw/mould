pub mod parameter;
use super::await_task;
use super::download_file;
use super::get_session;
use super::upload_file;
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
    context: &Context,
    append_log: &AppendLog,
    resource_index: u32,
) -> Result<(), String> {
    let context = context.clone();
    let append_log = append_log.clone();
    let result = await_task(&context.clone(), move || {
        try_handle(
            configuration,
            parameter,
            &context,
            &append_log,
            resource_index,
        )
    })
    .await?;
    return result;
}

fn try_handle(
    configuration: Value,
    parameter: Value,
    context: &Context,
    append_log: &AppendLog,
    resource_index: u32,
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
    append_log(LogLevel::Info, String::from("正在从服务器下载配置"));
    let value = download_file(&session, &remote_path)?;
    let remote_path_prefix = format!("远程文件路径：{}, ", remote_path);
    append_log(LogLevel::Info, String::from("正在解析配置"));
    let value =
        String::from_utf8(value).map_err(|err| format!("原始配置不是UTF-8格式字符串: {err}"))?;
    let root_value: Value = json5::from_str(&value)
        .map_err(|err| format!("{remote_path_prefix}解析json配置文件失败: {err}"))?;
    append_log(LogLevel::Info, String::from("正在修改配置"));
    let new_root_value = context.modify_json_custom(
        root_value,
        &parameter.json_path,
        &parameter.replace_function,
        resource_index,
    )?;
    append_log(LogLevel::Info, String::from("正在生成新配置文件"));
    let new_root_value = serde_json::to_vec_pretty(&new_root_value)
        .map_err(|err| format!("{remote_path_prefix}序列化json配置文件失败: {err}"))?;
    append_log(LogLevel::Info, String::from("正在上传新配置文件"));
    upload_file(&session, &remote_path, &new_root_value, 0o644)?;
    append_log(LogLevel::Info, String::from("修改成功!"));
    return Ok(());
}
