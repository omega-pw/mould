pub mod parameter;
use super::await_future;
use super::get_client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;
use tokio::fs::File;

pub async fn handle(
    configuration: Value,
    parameter: Value,
    context: &Context,
    append_log: &AppendLog,
) -> Result<(), String> {
    append_log(LogLevel::Info, String::from("正在构造s3客户端"));
    let (client, bucket) = get_client(configuration)?;
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    append_log(LogLevel::Info, String::from("正在获取文件"));
    let file = context.download_file(&parameter.file.key).await?;
    append_log(LogLevel::Info, String::from("获取文件完成"));
    let metadata = file
        .metadata()
        .map_err(|err| format!("获取文件metadata失败: {err}"))?;
    let content_length = metadata.len();
    let key = parameter.key;
    let mime_type = parameter.file.mime_type;
    let result = await_future(try_handle(
        client,
        file,
        bucket,
        content_length as i64,
        key,
        mime_type,
        append_log.clone(),
    ))
    .await?;
    result?;
    append_log(LogLevel::Info, String::from("上传成功!"));
    return Ok(());
}

async fn try_handle(
    client: Client,
    file: std::fs::File,
    bucket: String,
    content_length: i64,
    key: String,
    mime_type: String,
    append_log: AppendLog,
) -> Result<(), String> {
    let key_prefix = format!("key：{}, ", key);
    append_log(LogLevel::Info, String::from("正在构造上传请求"));
    let body = ByteStream::read_from()
        .file(File::from_std(file))
        .build()
        .await
        .map_err(|err| format!("{key_prefix}准备上传文件数据失败: {err}"))?;
    append_log(LogLevel::Info, String::from("正在上传文件到s3服务器"));
    let _resp = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_length(content_length as i64)
        .content_type(mime_type)
        .body(body)
        .send()
        .await
        .map_err(|err| format!("{key_prefix}上传文件失败: {err}"))?;
    return Ok(());
}
