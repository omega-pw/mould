use super::get_client;
use mould_extension_sdk::serde_json::Value;

pub async fn handle(configuration: Value) -> Result<(), String> {
    let _ = get_client(configuration).await?;
    return Ok(());
}
