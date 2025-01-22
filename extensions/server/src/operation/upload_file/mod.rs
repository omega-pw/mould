pub mod parameter;
use super::get_session;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;
use ssh2::Session;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    context: &Context,
    append_log: &AppendLog,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    append_log(LogLevel::Info, String::from("正在获取文件"));
    let file = context.download_file(&parameter.file.key).await?;
    append_log(LogLevel::Info, String::from("获取文件完成"));
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    let append_log = append_log.clone();
    context
        .spawn_blocking(Box::new(move || {
            let result = try_handle(configuration, parameter, file, &append_log);
            result_clone.lock().unwrap().replace(result);
        }))
        .await?;
    return result.lock().unwrap().take().unwrap();
}

fn try_handle(
    configuration: Value,
    parameter: Parameter,
    mut file: File,
    append_log: &AppendLog,
) -> Result<(), String> {
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
    append_log(LogLevel::Info, String::from("正在上传文件"));
    upload_file(&session, &remote_path, &mut file, 0o644)?;
    append_log(LogLevel::Info, String::from("上传文件成功!"));
    return Ok(());
}

fn upload_file(
    session: &Session,
    remote_path: &str,
    file: &mut File,
    mode: i32,
) -> Result<(), String> {
    let remote_path_prefix = format!("远程文件路径：{}, ", remote_path);
    let metadata = file
        .metadata()
        .map_err(|err| format!("获取文件metadata失败: {err}"))?;
    let mut remote_file = session
        .scp_send(&Path::new(remote_path), mode, metadata.len(), None)
        .map_err(|err| format!("{remote_path_prefix}通过SCP开始发送文件失败: {err}"))?;
    let mut buf = [0; 1024];
    loop {
        let count = file
            .read(&mut buf[..])
            .map_err(|err| format!("{remote_path_prefix}读取文件内容失败: {err}"))?;
        if 0 < count {
            remote_file
                .write(&buf[..count])
                .map_err(|err| format!("{remote_path_prefix}发送文件流数据失败: {err}"))?;
        } else {
            break;
        }
    }
    remote_file
        .send_eof()
        .map_err(|err| format!("{remote_path_prefix}发送文件结束信号失败: {err}"))?;
    remote_file
        .wait_eof()
        .map_err(|err| format!("{remote_path_prefix}等待结束信号确认失败: {err}"))?;
    remote_file
        .close()
        .map_err(|err| format!("{remote_path_prefix}关闭远程文件失败: {err}"))?;
    remote_file
        .wait_close()
        .map_err(|err| format!("{remote_path_prefix}关闭远程文件失败: {err}"))?;
    return Ok(());
}
