use super::await_future;
use super::get_client;
use crate::config::Config;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Context;

pub async fn handle(configuration: Value, _context: &Context) -> Result<(), String> {
    let configuration = Config::try_form_value(configuration)?;
    let result = await_future(try_handle(configuration)).await?;
    return result;
}

async fn try_handle(configuration: Config) -> Result<(), String> {
    let _ = get_client(&configuration).await?;
    return Ok(());
}
