pub mod upload_file;
use crate::config::Config;
use aws_sdk_s3::config::timeout::TimeoutConfig;
use aws_sdk_s3::config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, SharedCredentialsProvider};
use aws_sdk_s3::Client;
use aws_types::region::Region;
use aws_types::sdk_config::SdkConfig;
use futures::channel::oneshot;
pub mod test;
use aws_smithy_async::rt::sleep::TokioSleep;
use mould_extension_sdk::serde_json::Value;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

pub fn get_client(configuration: Value) -> Result<(Client, String), String> {
    let configuration = Config::try_form_value(configuration)?;
    let credentials = Credentials::new(
        configuration.access_key.to_string(),
        configuration.secret_key.to_string(),
        None,
        None,
        "",
    );
    let shared_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .endpoint_url(configuration.endpoint.to_string())
        .region(Region::new(configuration.region.to_string()))
        .timeout_config(
            TimeoutConfig::builder()
                .connect_timeout(Duration::from_secs(30))
                .build(),
        )
        .sleep_impl(Arc::new(TokioSleep::new()))
        //behavior_version参数必填，否则会报错
        .behavior_version(BehaviorVersion::latest())
        .build();
    let client = aws_sdk_s3::Client::new(&shared_config);
    return Ok((client, configuration.bucket));
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
