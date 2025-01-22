pub mod execute;
pub mod test;
use crate::config::Config;
use futures::channel::oneshot;
use mysql_async::Conn;
use mysql_async::OptsBuilder;
use mysql_async::Pool;
use mysql_async::PoolConstraints;
use mysql_async::PoolOpts;
use mysql_async::SslOpts;
use std::fmt::Debug;
use std::future::Future;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn get_conn(configuration: Config, root_cert: Option<File>) -> Result<Conn, String> {
    let host_prefix = format!(
        "主机：{}, 端口：{}, ",
        configuration.host, configuration.port
    );
    let builder = OptsBuilder::default()
        .ip_or_hostname(configuration.host)
        .tcp_port(configuration.port)
        .user(Some(configuration.user))
        .pass(Some(configuration.password))
        .db_name(Some(configuration.dbname))
        .ssl_opts(if configuration.ssl {
            let mut ssl_opts = SslOpts::default();
            if let Some(mut root_cert) = root_cert {
                let mut buf = Vec::new();
                root_cert
                    .read_to_end(&mut buf)
                    .await
                    .map_err(|err| format!("{host_prefix}读取ssl根证书失败: {err}"))?;
                ssl_opts = ssl_opts.with_root_certs(vec![buf.into()]);
            }
            Some(ssl_opts)
        } else {
            None
        })
        .pool_opts(PoolOpts::default().with_constraints(PoolConstraints::new(0, 1).unwrap()));
    let pool = Pool::new(builder);
    let conn_ret = pool
        .get_conn()
        .await
        .map_err(|err| format!("{host_prefix}连接mysql数据库失败: {err}"));
    return conn_ret;
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
