pub mod parameter;
use super::await_future;
use super::get_conn;
use crate::config::Config;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use mysql_async::prelude::Queryable;
use parameter::Parameter;
use tokio::fs::File;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    context: &Context,
    append_log: &AppendLog,
) -> Result<(), String> {
    let configuration = Config::try_form_value(configuration)?;
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    let root_cert = if let Some(root_cert) = configuration.root_cert.as_ref() {
        append_log(LogLevel::Info, String::from("正在下载根证书"));
        let root_cert = context.download_file(&root_cert.key).await?;
        Some(tokio::fs::File::from_std(root_cert))
    } else {
        None
    };
    let sql = parameter.sql;
    let result = await_future(try_handle(
        configuration,
        root_cert,
        sql,
        append_log.clone(),
    ))
    .await?;
    return result;
}

async fn try_handle(
    configuration: Config,
    root_cert: Option<File>,
    sql: String,
    append_log: AppendLog,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在连接mysql服务器"));
    let mut client = get_conn(configuration, root_cert).await?;
    append_log(LogLevel::Info, String::from("连接mysql服务器成功"));
    append_log(LogLevel::Info, String::from("正在执行sql"));
    client
        .query_drop(&sql)
        .await
        .map_err(|err| format!("执行sql语句失败: {err}, sql: {sql}"))?;
    append_log(LogLevel::Info, String::from("执行sql成功!"));
    return Ok(());
}
