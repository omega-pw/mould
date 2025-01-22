use super::await_task;
use super::get_session;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Context;

pub async fn handle(configuration: Value, context: &Context) -> Result<(), String> {
    let result = await_task(&context.clone(), move || try_handle(configuration)).await?;
    return result;
}

fn try_handle(configuration: Value) -> Result<(), String> {
    let _ = get_session(configuration)?;
    return Ok(());
}
