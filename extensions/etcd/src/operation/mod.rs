pub mod modify_json;
pub mod modify_json_custom;
pub mod put;
pub mod test;
use crate::config::Config;
use etcd_rs::Client;
use etcd_rs::ClientConfig;
use futures::channel::oneshot;
use mould_extension_sdk::serde_json::Value;
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

pub async fn get_client(configuration: Value) -> Result<Client, String> {
    let configuration = Config::try_form_value(configuration)?;
    let endpoints_prefix = format!("endpoints：{:?}, ", configuration.endpoints);
    let mut client_config = ClientConfig::new(configuration.endpoints);
    client_config.auth = configuration.auth;
    client_config.connect_timeout = Duration::from_secs(30);
    let client = Client::connect(client_config)
        .await
        .map_err(|err| format!("{endpoints_prefix}连接etcd服务器失败: {err}"))?;
    return Ok(client);
}

pub async fn await_future<O: Send + Debug + 'static>(
    future: impl Future<Output = O> + Send + 'static,
) -> Result<O, String> {
    let (sender, receiver) = oneshot::channel::<O>();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let output = future.await;
            sender.send(output).unwrap();
        });
    });
    receiver.await.map_err(|err| err.to_string())
}
