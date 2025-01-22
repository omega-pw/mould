use super::await_future;
use super::get_client;
use crate::config::Config;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Context;
use tokio::fs::File;

pub async fn handle(configuration: Value, context: &Context) -> Result<(), String> {
    let configuration = Config::try_form_value(configuration)?;
    let root_cert = if let Some(root_cert) = configuration.root_cert.as_ref() {
        let root_cert = context.download_file(&root_cert.key).await?;
        Some(tokio::fs::File::from_std(root_cert))
    } else {
        None
    };
    let result = await_future(try_handle(configuration, root_cert)).await?;
    return result;
}

async fn try_handle(configuration: Config, root_cert: Option<File>) -> Result<(), String> {
    let _ = get_client(configuration, root_cert).await?;
    return Ok(());
}
