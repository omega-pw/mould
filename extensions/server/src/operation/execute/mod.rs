pub mod parameter;
use super::await_task;
use super::download_file;
use super::get_session;
use super::upload_file;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;
use ssh2::Session;
use std::io::prelude::*;
use uuid::Uuid;

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
    let task_id = Uuid::new_v4().to_string();
    let tmp_dir = "/tmp/mould-server-extension";
    let script_path = format!("{}/task-{}.sh", tmp_dir, task_id);
    let log_path = format!("{}/task-{}.log", tmp_dir, task_id);
    let shell = parameter.shell;
    let script = if parameter.script.starts_with("#!/bin/") {
        parameter.script.clone()
    } else {
        format!("#!{}\n{}", shell, parameter.script)
    };
    let sudo_prefix = if parameter.sudo {
        format!("echo {} | sudo -S ", config.password)
    } else {
        String::from("")
    };
    let command = if config.workspace.is_empty() {
        format!("{sudo_prefix}{shell} {script_path} > {log_path}")
    } else {
        let workspace = config.workspace;
        format!("cd {workspace} && {sudo_prefix}{shell} {script_path} > {log_path}")
    };
    append_log(LogLevel::Info, String::from("正在服务器上准备工作目录"));
    exec_remote(&session, &format!("mkdir -p {}", tmp_dir), true)
        .map_err(|err| format!("准备工作目录失败, {}", err))?;
    append_log(LogLevel::Info, String::from("正在上传脚本"));
    upload_file(&session, &script_path, script.as_bytes(), 0o754)
        .map_err(|err| format!("上传脚本失败, {}", err))?;
    append_log(LogLevel::Info, String::from("正在执行脚本"));
    exec_remote(&session, &command, false).map_err(|err| format!("执行脚本失败, {}", err))?;
    append_log(LogLevel::Info, String::from("正在下载脚本输出日志"));
    let output =
        download_file(&session, &log_path).map_err(|err| format!("下载脚本日志失败, {}", err))?;
    append_log(
        LogLevel::Info,
        String::from("正在清理脚本文件和脚本输出日志"),
    );
    exec_remote(
        &session,
        &format!("rm -rf {} {}", script_path, log_path),
        true,
    )
    .map_err(|err| format!("移除脚本和日志失败, {}", err))?;
    append_log(
        LogLevel::Info,
        format!(
            "执行脚本完毕，脚本输出日志:\n{}",
            String::from_utf8_lossy(&output)
        ),
    );
    return Ok(());
}

fn exec_remote(
    session: &Session,
    command: &str,
    print_cmd_on_error: bool,
) -> Result<String, String> {
    let cmd_prefix = if print_cmd_on_error {
        format!("命令：{command}, ")
    } else {
        String::from("")
    };
    let mut output = String::new();
    let mut channel = session
        .channel_session()
        .map_err(|err| format!("{cmd_prefix}ssh创建通道失败: {err}"))?;
    channel
        .exec(command)
        .map_err(|err| format!("{cmd_prefix}执行远程命令失败: {err}"))?;
    channel
        .read_to_string(&mut output)
        .map_err(|err| format!("{cmd_prefix}读取远程命令输出结果失败: {err}"))?;
    let exit_status = channel
        .exit_status()
        .map_err(|err| format!("{cmd_prefix}获取远程命令退出码失败: {err}"))?;
    if 0 == exit_status {
        return Ok(output);
    } else {
        let mut error_message = String::new();
        channel
            .stderr()
            .read_to_string(&mut error_message)
            .map_err(|err| format!("{cmd_prefix}读取远程命令错误信息失败: {err}"))?;
        let error_message =
            format!("{cmd_prefix}结果：{output}, 错误码：{exit_status}, 错误：{error_message}");
        return Err(error_message);
    }
}
