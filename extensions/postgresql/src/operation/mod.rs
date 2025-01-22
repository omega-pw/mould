pub mod execute;
pub mod test;
use crate::config::Config;
use futures::channel::oneshot;
use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_postgres::config::SslMode;
use tokio_postgres::{Client, Config as DbConfig, NoTls};

pub async fn get_client(configuration: Config, root_cert: Option<File>) -> Result<Client, String> {
    let mut cfg = DbConfig::new();
    cfg.host(&configuration.host);
    cfg.port(configuration.port);
    cfg.dbname(&configuration.dbname);
    cfg.user(&configuration.user);
    cfg.password(&configuration.password);
    cfg.connect_timeout(Duration::from_secs(30));
    if configuration.ssl {
        cfg.ssl_mode(SslMode::Require);
    }
    let host_prefix = format!(
        "主机：{}, 端口：{}, ",
        configuration.host, configuration.port
    );
    let client = if configuration.ssl {
        let mut builder = TlsConnector::builder();
        if let Some(mut root_cert) = root_cert {
            let mut buf = Vec::new();
            root_cert
                .read_to_end(&mut buf)
                .await
                .map_err(|err| format!("{host_prefix}读取ssl根证书失败: {err}"))?;
            let root_cert = Certificate::from_pem(&buf)
                .map_err(|err| format!("{host_prefix}解析ssl根证书失败: {err}"))?;
            builder.add_root_certificate(root_cert);
        }
        let connector = builder
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|err| format!("{host_prefix}创建ssl连接器失败: {err}"))?;
        let (client, connection) = cfg
            .connect(MakeTlsConnector::new(connector))
            .await
            .map_err(|err| format!("{host_prefix}连接postgres数据库失败: {err}"))?;
        //直接用tokio::spawn启动connection无法结束，原因尚不清楚
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(connection).ok();
        });
        client
    } else {
        let (client, connection) = cfg
            .connect(NoTls)
            .await
            .map_err(|err| format!("{host_prefix}连接postgres数据库失败: {err}"))?;
        //直接用tokio::spawn启动connection无法结束，原因尚不清楚
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(connection).ok();
        });
        client
    };
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
