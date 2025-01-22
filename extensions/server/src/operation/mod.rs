pub mod execute;
pub mod modify_json;
pub mod modify_json_custom;
pub mod put;
pub mod upload_file;
use crate::config::Config;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Context;
use ssh2::Session;
use std::fmt::Debug;
use std::io::prelude::*;
use std::net::TcpStream;
pub mod test;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub fn get_session(configuration: Value) -> Result<(Session, Config), String> {
    let configuration = Config::try_form_value(configuration)?;
    let host_prefix = format!(
        "主机：{}, 端口：{}, ",
        configuration.host, configuration.port
    );
    let tcp = TcpStream::connect_timeout(
        &format!("{}:{}", configuration.host, configuration.port)
            .parse()
            .map_err(|err| format!("{host_prefix}解析主机地址失败: {err}"))?,
        Duration::from_secs(30),
    )
    .map_err(|err| format!("{host_prefix}连接远程服务器失败: {err}"))?;
    let mut session =
        Session::new().map_err(|err| format!("{host_prefix}创建ssh会话失败: {err}"))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|err| format!("{host_prefix}ssh会话握手失败: {err}"))?;
    if configuration.private_key.is_empty() {
        session
            .userauth_password(&configuration.user, &configuration.password)
            .map_err(|err| format!("{host_prefix}ssh会话认证失败: {err}"))?;
    } else {
        let public_key = if configuration.public_key.is_empty() {
            None
        } else {
            Some(configuration.public_key.as_str())
        };
        let passphrase = if configuration.passphrase.is_empty() {
            None
        } else {
            Some(configuration.passphrase.as_str())
        };
        session
            .userauth_pubkey_memory(
                &configuration.user,
                public_key,
                &configuration.private_key,
                passphrase,
            )
            .map_err(|err| format!("{host_prefix}ssh会话认证失败: {err}"))?;
    }

    if session.authenticated() {
        return Ok((session, configuration));
    } else {
        return Err(format!("{host_prefix}授权登录远程服务器失败"));
    }
}

pub async fn await_task<O: Send + Debug + 'static>(
    context: &Context,
    task: impl FnOnce() -> O + Send + 'static,
) -> Result<O, String> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    context
        .spawn_blocking(Box::new(move || {
            let ret = task();
            result_clone.lock().unwrap().replace(ret);
        }))
        .await?;
    return Ok(result.lock().unwrap().take().unwrap());
}

fn download_file(session: &Session, remote_path: &str) -> Result<Vec<u8>, String> {
    let remote_path_prefix = format!("远程路径：{}, ", remote_path);
    let (mut remote_file, _stat) = session
        .scp_recv(Path::new(remote_path))
        .map_err(|err| format!("{remote_path_prefix}通过SCP开始下载文件失败: {err}"))?;
    let mut content = Vec::new();
    remote_file
        .read_to_end(&mut content)
        .map_err(|err| format!("{remote_path_prefix}读取远程文件流数据失败: {err}"))?;
    remote_file
        .send_eof()
        .map_err(|err| format!("{remote_path_prefix}发送文件结束信号失败: {err}"))?;
    remote_file
        .wait_eof()
        .map_err(|err| format!("{remote_path_prefix}等待结束信号确认失败: {err}"))?;
    remote_file
        .close()
        .map_err(|err| format!("{remote_path_prefix}关闭远程文件失败: {err}"))?;
    remote_file
        .wait_close()
        .map_err(|err| format!("{remote_path_prefix}关闭远程文件失败: {err}"))?;
    return Ok(content);
}

fn upload_file(
    session: &Session,
    remote_path: &str,
    content: &[u8],
    mode: i32,
) -> Result<(), String> {
    let remote_path_prefix = format!("远程文件路径：{}, ", remote_path);
    let mut remote_file = session
        .scp_send(&Path::new(remote_path), mode, content.len() as u64, None)
        .map_err(|err| format!("{remote_path_prefix}通过SCP开始发送文件失败: {err}"))?;
    remote_file
        .write(content)
        .map_err(|err| format!("{remote_path_prefix}发送文件流数据失败: {err}"))?;
    remote_file
        .send_eof()
        .map_err(|err| format!("{remote_path_prefix}发送文件结束信号失败: {err}"))?;
    remote_file
        .wait_eof()
        .map_err(|err| format!("{remote_path_prefix}等待结束信号确认失败: {err}"))?;
    remote_file
        .close()
        .map_err(|err| format!("{remote_path_prefix}关闭远程文件失败: {err}"))?;
    remote_file
        .wait_close()
        .map_err(|err| format!("{remote_path_prefix}关闭远程文件失败: {err}"))?;
    return Ok(());
}
