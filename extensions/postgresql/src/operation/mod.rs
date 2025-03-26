pub mod execute;
pub mod test;
use crate::config::Config;
use futures::channel::oneshot;
use rustls::ClientConfig;
use rustls::RootCertStore;
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_postgres::config::SslMode;
use tokio_postgres::{Client, Config as DbConfig, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

fn load_native_certs() -> Result<RootCertStore, String> {
    let mut root_cert_store = RootCertStore::empty();
    let mut certs_result = rustls_native_certs::load_native_certs();
    if let Some(err) = certs_result.errors.pop() {
        return Err(err.to_string());
    }
    root_cert_store.add_parsable_certificates(certs_result.certs);
    return Ok(root_cert_store);
}

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
        let root_cert_store = if let Some(mut root_cert) = root_cert {
            let mut buf = Vec::new();
            root_cert
                .read_to_end(&mut buf)
                .await
                .map_err(|err| format!("{host_prefix}读取ssl根证书失败: {err}"))?;
            let certs = rustls_pemfile::certs(&mut buf.as_ref())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| format!("{host_prefix}解析ssl根证书失败: {err}"))?;
            let mut root_cert_store = RootCertStore::empty();
            root_cert_store.add_parsable_certificates(certs);
            root_cert_store
        } else {
            load_native_certs()?
        };
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = MakeRustlsConnect::new(config);
        let (client, connection) = cfg
            .connect(connector)
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
