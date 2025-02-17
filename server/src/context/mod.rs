use super::config::CacheServer;
use super::config::Config;
use super::config::DataSource;
use super::config::Oauth2Server;
use super::config::OpenidServer;
use crate::native_common;
use crate::sdk;
use aws_sdk_s3::config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::config::SharedCredentialsProvider;
use aws_sdk_s3::Client;
use aws_types::region::Region;
use aws_types::sdk_config::SdkConfig;
use deadpool::managed::Object;
use deadpool_postgres::{Manager, Pool};
use deadpool_redis;
use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection;
use deadpool_redis::ConnectionAddr;
use deadpool_redis::ConnectionInfo;
use deadpool_redis::RedisConnectionInfo;
use log;
use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use mould_extension_sdk::pluginator;
use mould_extension_sdk::ContextTrait;
use mould_extension_sdk::Extension;
use native_common::cache::RedisCache;
use sdk::storage::UPLOAD_API;
// use native_common::utils::Snowflake;
use native_tls::{Certificate, TlsConnector};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use object_storage_lib::Config as OssConfig;
use object_storage_lib::Oss;
use object_storage_lib::OssHandler;
use openid::{Client as OpenidClient, DiscoveredClient};
use pluginator::LoadedPlugin;
use postgres_native_tls::MakeTlsConnector;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryInto;
use std::future::Future;
use std::io::SeekFrom;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tihu::Id;
use tihu::LightString;
use tihu_native::http::HttpHandler;
use tihu_native::ErrNo;
use tokio::fs::create_dir;
use tokio::fs::read_dir;
use tokio::fs::try_exists;
use tokio::fs::File;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio_postgres::config::SslMode;
use tokio_postgres::{Config as DbConfig, NoTls};
use url::Url;
use uuid::Uuid;

mod extension;

mould_extension_sdk::plugin_trait!(Extension);

pub const RPC_TIMEOUT: u64 = 10;

pub struct ExtensionContext {
    pub oss_client: Arc<Client>,
    pub bucket: LightString,
}

#[async_trait::async_trait]
impl ContextTrait for ExtensionContext {
    async fn spawn_blocking(&self, task: Box<dyn FnOnce() + Send + 'static>) -> Result<(), String> {
        tokio::task::spawn_blocking(task)
            .await
            .map_err(|err| err.to_string())?;
        return Ok(());
    }
    fn spawn_future(
        &self,
        future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    ) -> Result<(), String> {
        tokio::spawn(future);
        return Ok(());
    }
    fn modify_json_custom(
        &self,
        target: Value,
        json_path: &str,
        js_func: &str,
        resource_index: u32,
    ) -> Result<Value, String> {
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(scope, Default::default());
        let mut error_opt = None;
        let scope = &mut v8::ContextScope::new(scope, context);
        let new_target = jsonpath_lib::replace_with(target, json_path, &mut |old_val| {
            if error_opt.is_none() {
                match try_replace(scope, js_func, old_val, resource_index) {
                    Ok(new_val) => Some(new_val),
                    Err(error) => {
                        error_opt.replace(error);
                        None
                    }
                }
            } else {
                None
            }
        })
        .map_err(|err| err.to_string())?;
        if let Some(error) = error_opt {
            return Err(error);
        }
        return Ok(new_target);
    }
    fn modify_json(
        &self,
        target: Value,
        json_path: &str,
        new_value: Value,
    ) -> Result<Value, String> {
        let new_target =
            jsonpath_lib::replace_with(target, json_path, &mut |_old_val| Some(new_value.clone()))
                .map_err(|err| err.to_string())?;
        return Ok(new_target);
    }
    async fn download_file(&self, key: &str) -> Result<std::fs::File, String> {
        let resp = self
            .oss_client
            .get_object()
            .bucket(self.bucket.to_string())
            .key(key)
            .send()
            .await
            .map_err(|err| {
                log::error!("下载文件失败: {:?}", err);
                err.to_string()
            })?;
        let temp_dir = tempfile::env::temp_dir();
        if !temp_dir.exists() {
            std::fs::create_dir(temp_dir).map_err(|err| {
                log::error!("临时文件夹不存在，并且创建失败: {:?}", err);
                err.to_string()
            })?;
        }
        let tmp_file = tempfile::tempfile().map_err(|err| {
            log::error!("创建临时文件失败: {:?}", err);
            err.to_string()
        })?;
        let mut tmp_file = File::from_std(tmp_file);
        let mut stream = resp.body;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|err| {
                log::error!("下载对象数据失败: {:?}", err);
                err.to_string()
            })?;
            tmp_file.write_all(&chunk).await.map_err(|err| {
                log::error!("写入数据到临时文件失败: {:?}", err);
                err.to_string()
            })?;
        }
        tmp_file.flush().await.map_err(|err| {
            log::error!("刷新数据存储失败: {:?}", err);
            err.to_string()
        })?;
        tmp_file.seek(SeekFrom::Start(0)).await.map_err(|err| {
            log::error!("重置游标失败: {:?}", err);
            err.to_string()
        })?;
        let tmp_file = tmp_file.into_std().await;
        return Ok(tmp_file);
    }
}

fn try_replace(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    js_func: &str,
    old_val: Value,
    resource_index: u32,
) -> Result<Value, String> {
    let old_val = serde_json::to_string(&old_val).map_err(|err| err.to_string())?;
    let replace_script = format!(
        r#"JSON.stringify(({})({},{}))"#,
        js_func, old_val, resource_index
    );
    let code =
        v8::String::new(scope, &replace_script).ok_or_else(|| String::from("构建js脚本失败"))?;
    let script =
        v8::Script::compile(scope, code, None).ok_or_else(|| String::from("编译js脚本失败"))?;
    let result = script
        .run(scope)
        .ok_or_else(|| String::from("运行js脚本失败"))?;
    let result = result
        .to_string(scope)
        .ok_or_else(|| String::from("获取运行结果失败"))?;
    let result = result.to_rust_string_lossy(scope);
    return serde_json::from_str(&result).map_err(|err| err.to_string());
}

fn adjust_oss_error_code(error_code: i32) -> i32 {
    if 0 >= error_code {
        return error_code;
    }
    // 01xxxx表示对象存储的错误
    return 10000 + error_code;
}

pub struct Context {
    pub config: Arc<Config>,
    // snowflake: Snowflake,
    cache_pool: Arc<deadpool_redis::Pool>,
    db_pool: Arc<Pool>,
    oss_client: Arc<Client>,
    oss_handler: Arc<dyn HttpHandler>,
    extensions: Arc<Vec<(sdk::extension::Extension, Arc<LoadedPlugin<dyn Extension>>)>>,
    extension_context: mould_extension_sdk::Context,
    oauth2_clients: HashMap<String, (Arc<BasicClient>, Oauth2Server)>,
    openid_clients: HashMap<String, (Arc<OpenidClient>, OpenidServer)>,
}

impl Context {
    pub async fn try_init_from_config(config: Config) -> Result<Context, anyhow::Error> {
        if let Some(log_cfg_path) = config.log_cfg_path.as_ref() {
            if let Err(err) = log4rs::init_file(log_cfg_path.as_ref(), Default::default()) {
                println!("init log4rs failed, {}", err);
                init_console_log()?;
            }
        } else {
            init_console_log()?;
        }
        if !try_exists(&config.job_log_dir).await? {
            create_dir(&config.job_log_dir).await?;
        }
        let extensions = load_extensions(&config.extension_dir).await?;
        // let snowflake = Snowflake::new(0, None);
        let cache_pool = init_cache_pool(&config.cache_server)?;
        let db_pool = init_db_pool(&config.data_source)?;
        let bucket = config.oss.bucket.clone();
        let mut oss_handler = OssHandler::try_init_from_config(
            OssConfig {
                oss: config.oss.clone(),
            },
            adjust_oss_error_code,
        )
        .await?;
        oss_handler
            .add_get_mapping(
                LightString::from_static("/blob/"),
                LightString::from_static("blob/"),
            )
            .add_get_mapping(
                LightString::from_static("/file/"),
                LightString::from_static("file/"),
            )
            .add_upload_mapping(
                LightString::from_static(UPLOAD_API),
                LightString::from_static("blob/"),
            );
        let oss_client = Arc::new(init_oss_client(&config.oss)?);
        let mut oauth2_clients = HashMap::with_capacity(config.oauth2_servers.len());
        for (provider, oauth2_server) in &config.oauth2_servers {
            let redirect_uri = format!("{}/oauth2/authorize/{}", config.public_path, provider);
            let oauth2_client = Arc::new(
                BasicClient::new(
                    ClientId::new(oauth2_server.client_id.clone()),
                    Some(ClientSecret::new(oauth2_server.client_secret.clone())),
                    AuthUrl::new(oauth2_server.auth_url.clone())?,
                    Some(TokenUrl::new(oauth2_server.token_url.clone())?),
                )
                .set_redirect_uri(RedirectUrl::new(redirect_uri)?),
            );
            oauth2_clients.insert(provider.clone(), (oauth2_client, oauth2_server.clone()));
        }
        let mut openid_clients = HashMap::with_capacity(config.openid_servers.len());
        for (provider, openid_server) in &config.openid_servers {
            let redirect_uri = format!("{}/oidc/authorize/{}", config.public_path, provider);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(RPC_TIMEOUT))
                .build()?;
            let openid_client = Arc::new(
                DiscoveredClient::discover_with_client(
                    client,
                    openid_server.client_id.clone(),
                    openid_server.client_secret.clone(),
                    redirect_uri,
                    openid_server.issuer.as_str().try_into()?,
                )
                .await?,
            );
            openid_clients.insert(provider.clone(), (openid_client, openid_server.clone()));
        }
        let context = Context {
            config: Arc::new(config),
            // snowflake: snowflake,
            cache_pool: Arc::new(cache_pool),
            db_pool: Arc::new(db_pool),
            oss_client: oss_client.clone(),
            oss_handler: Arc::new(oss_handler),
            extensions: Arc::new(extensions),
            extension_context: Arc::new(ExtensionContext {
                oss_client: oss_client,
                bucket: bucket,
            }),
            oauth2_clients: oauth2_clients,
            openid_clients: openid_clients,
        };
        return Ok(context);
    }

    // pub async fn get_session_timeout(&self) -> Result<u8, ErrNo> {
    //     return Ok(self.config.session_timeout);
    // }

    pub async fn get_sign_secret(&self) -> Result<Arc<Vec<u8>>, ErrNo> {
        return Ok(self.config.sign_secret.clone());
    }

    pub async fn get_server_random_value(&self) -> Result<[u8; 32], ErrNo> {
        return Ok(self.config.server_random_value.clone());
    }

    pub async fn get_rsa_pub_key(&self) -> Result<RsaPublicKey, ErrNo> {
        return Ok(self.config.rsa_pub_key.clone());
    }

    pub async fn get_rsa_pri_key(&self) -> Result<RsaPrivateKey, ErrNo> {
        return Ok(self.config.rsa_pri_key.clone());
    }

    pub async fn get_rsa_pub_key_content(&self) -> Result<LightString, ErrNo> {
        return Ok(self.config.rsa_pub_key_content.clone());
    }

    pub async fn get_cache_pool(&self) -> Option<Arc<deadpool_redis::Pool>> {
        return Some(self.cache_pool.clone());
    }

    pub async fn get_cache_client(&self) -> Result<Connection, ErrNo> {
        let pool = self.get_cache_pool().await.ok_or(ErrNo::NoCacheClient)?;
        pool.get().await.map_err(|err| ErrNo::Other(err.into()))
    }

    pub async fn get_cache_mgr(&self) -> Result<Arc<RedisCache>, ErrNo> {
        return Ok(Arc::new(RedisCache::new(self.get_cache_pool().await)));
    }

    pub async fn get_db_client(&self) -> Result<Object<deadpool_postgres::Manager>, ErrNo> {
        self.db_pool
            .get()
            .await
            .map_err(|err| ErrNo::Other(err.into()))
    }

    pub fn get_oss_client(&self) -> Arc<Client> {
        return self.oss_client.clone();
    }

    pub fn get_bucket(&self) -> LightString {
        return self.config.oss.bucket.clone();
    }

    pub fn new_id(&self) -> Id {
        Uuid::now_v7()
    }

    pub fn get_oss_handler(&self) -> &Arc<dyn HttpHandler> {
        return &self.oss_handler;
    }

    pub fn get_extension(&self, extension_id: &str) -> Option<Arc<LoadedPlugin<dyn Extension>>> {
        for extension in self.extensions.iter() {
            if extension.0.id == extension_id {
                return Some(extension.1.clone());
            }
        }
        return None;
    }

    pub fn get_extension_info(
        &self,
        extension_id: &str,
    ) -> Option<&(sdk::extension::Extension, Arc<LoadedPlugin<dyn Extension>>)> {
        for extension in self.extensions.iter() {
            if extension.0.id == extension_id {
                return Some(&extension);
            }
        }
        return None;
    }

    pub fn get_extensions(&self) -> Vec<sdk::extension::Extension> {
        return self
            .extensions
            .iter()
            .map(|(extension, _)| extension.clone())
            .collect();
    }

    pub fn get_extension_context(&self) -> &mould_extension_sdk::Context {
        return &self.extension_context;
    }

    pub fn get_oauth2_client(
        &self,
        provider: &str,
    ) -> Result<(&Arc<BasicClient>, &Oauth2Server), ErrNo> {
        let (oauth2_client, oauth2_server) =
            self.oauth2_clients.get(provider).ok_or_else(|| {
                ErrNo::CommonError(LightString::from(format!(
                    "没有对应的oauth2 provider: {}",
                    provider,
                )))
            })?;
        return Ok((oauth2_client, oauth2_server));
    }
    pub fn get_openid_client(
        &self,
        provider: &str,
    ) -> Result<(&Arc<OpenidClient>, &OpenidServer), ErrNo> {
        let (openid_client, openid_server) =
            self.openid_clients.get(provider).ok_or_else(|| {
                ErrNo::CommonError(LightString::from(format!(
                    "没有对应的oidc provider: {}",
                    provider,
                )))
            })?;
        return Ok((openid_client, openid_server));
    }
}

fn init_console_log() -> Result<(), anyhow::Error> {
    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {l} {t}: {m}{n}",
        )))
        .build();
    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("CONSOLE", Box::new(console)))
        .build(Root::builder().appender("CONSOLE").build(LevelFilter::Warn))?;
    log4rs::init_config(config)?;
    return Ok(());
}

fn init_cache_pool(cache_server: &CacheServer) -> Result<deadpool_redis::Pool, RedisError> {
    let cache_connection_info = ConnectionInfo {
        addr: ConnectionAddr::Tcp(cache_server.host.clone(), cache_server.port),
        redis: RedisConnectionInfo {
            db: 0,
            username: cache_server.user.clone(),
            password: cache_server.password.clone(),
            protocol: Default::default(),
        },
    };
    let cache_mgr = deadpool_redis::Manager::new(cache_connection_info)?;
    let max_size = cache_server.max_size.unwrap_or(10).max(1);
    let cache_pool = deadpool_redis::Pool::builder(cache_mgr)
        .max_size(max_size)
        .build()
        .unwrap();
    return Ok(cache_pool);
}

fn init_db_pool(data_source: &DataSource) -> Result<Pool, native_tls::Error> {
    let mut cfg = DbConfig::new();
    cfg.host(&data_source.host);
    cfg.port(data_source.port);
    cfg.dbname(&data_source.dbname);
    cfg.user(&data_source.user);
    cfg.password(&data_source.password);
    if data_source.ssl.is_some() {
        cfg.ssl_mode(SslMode::Require);
    }
    let max_size = data_source.max_size.unwrap_or(2).max(1);
    let pool = if let Some(ssl_cfg) = &data_source.ssl {
        let mut builder = TlsConnector::builder();
        if let Some(root_cert) = ssl_cfg.root_cert.as_ref() {
            let root_cert = Certificate::from_pem(root_cert.as_bytes())?;
            builder.add_root_certificate(root_cert);
        }
        let connector = builder.danger_accept_invalid_certs(true).build()?;
        let connector = MakeTlsConnector::new(connector);
        let mgr = Manager::new(cfg, connector);
        Pool::builder(mgr).max_size(max_size).build().unwrap()
    } else {
        let mgr = Manager::new(cfg, NoTls);
        Pool::builder(mgr).max_size(max_size).build().unwrap()
    };
    return Ok(pool);
}

fn init_oss_client(oss: &Oss) -> Result<Client, http::uri::InvalidUri> {
    let credentials = Credentials::new(
        oss.access_key.to_string(),
        oss.secret_key.to_string(),
        None,
        None,
        "",
    );
    let shared_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .endpoint_url(oss.endpoint.as_str())
        .region(Region::new(oss.region.to_string()))
        //behavior_version参数必填，否则会报错
        .behavior_version(BehaviorVersion::latest())
        .build();
    let client = aws_sdk_s3::Client::new(&shared_config);
    return Ok(client);
}

async fn load_extensions(
    extension_dir: &str,
) -> Result<Vec<(sdk::extension::Extension, Arc<LoadedPlugin<dyn Extension>>)>, anyhow::Error> {
    let mut extensions = Vec::new();
    let mut entrys = read_dir(extension_dir).await?;
    let mut extension_files = Vec::new();
    while let Some(entry) = entrys.next_entry().await? {
        if let Ok(file_type) = entry.file_type().await {
            if file_type.is_file() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "dll" || extension == "so" {
                        extension_files.push(path);
                    }
                }
            }
        } else {
            println!("Couldn't get file type for {:?}", entry.path());
        }
    }
    for extension_file in extension_files {
        let extension = unsafe { load_plugin(extension_file) }.unwrap();
        extensions.push((
            extension::get_extension_info(extension.deref()),
            Arc::new(extension),
        ));
    }
    return Ok(extensions);
}
