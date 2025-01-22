pub mod modify_config_map_json;
pub mod modify_config_map_json_custom;
pub mod put_config_map;
pub mod restart_deployment;
use crate::config::Config;
use futures::channel::oneshot;
use std::fmt::Debug;
use std::future::Future;
pub mod set_image;
pub mod test;
use kube::client::ClientBuilder;
use kube::config::KubeConfigOptions;
use kube::config::Kubeconfig;
use kube::Client;
use kube::Config as KubeConfig;
use std::convert::TryFrom;

pub async fn get_client(configuration: &Config) -> Result<Client, String> {
    let kubeconfig = Kubeconfig::from_yaml(&configuration.kubeconfig)
        .map_err(|err| format!("kube配置格式不正确: {err}"))?;
    let config = KubeConfig::from_custom_kubeconfig(
        kubeconfig,
        &KubeConfigOptions {
            context: configuration.context.clone(),
            cluster: configuration.cluster.clone(),
            user: configuration.user.clone(),
        },
    )
    .await
    .map_err(|err| format!("构建kube配置失败: {err}"))?;
    let builder =
        ClientBuilder::try_from(config).map_err(|err| format!("构建kube配置失败: {err}"))?;
    let client = builder.build();
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
