use super::await_future;
use super::get_client;
use aws_sdk_s3::Client;
use mould_extension_sdk::serde_json::Value;

pub async fn handle(configuration: Value) -> Result<(), String> {
    let (client, bucket) = get_client(configuration)?;
    let result = await_future(try_handle(client, bucket)).await?;
    return result;
}

async fn try_handle(client: Client, bucket: String) -> Result<(), String> {
    let _resp = client
        .head_bucket()
        .bucket(bucket)
        .send()
        .await
        .map_err(|err| format!("获取桶信息失败: {err}"))?;
    return Ok(());
}
