#![allow(non_snake_case)]
pub mod parameter;
use super::await_future;
use crate::config::Config;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures::lock::Mutex;
use headers::{Authorization, HeaderMapExt};
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;
use reqwest::multipart::Part;
use reqwest::Method;
use reqwest::Request;
use reqwest::RequestBuilder;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::io::Read;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

static CLOUDFLARE_API_BASE_URL: &str = "https://api.cloudflare.com/client/v4";
const MAX_UPLOAD_ATTEMPTS: u32 = 5;
const MAX_DEPLOYMENT_ATTEMPTS: u32 = 3;
const UNAUTHORIZED: f64 = 8000013.0;
const UNKNOWN_ERROR: f64 = 8000000.0;

pub async fn try_max_times<D, E, F, T>(task: T, max_times: NonZeroU32) -> Result<D, E>
where
    F: Future<Output = Result<D, E>>,
    T: Fn(Option<E>) -> F,
{
    let extra_times = max_times.get() as usize - 1;
    match task(None).await {
        Ok(ret) => {
            return Ok(ret);
        }
        Err(mut error) => {
            for _ in 0..extra_times {
                match task(Some(error)).await {
                    Ok(ret) => {
                        return Ok(ret);
                    }
                    Err(latest_err) => {
                        error = latest_err;
                    }
                }
            }
            return Err(error);
        }
    }
}

pub async fn handle(
    configuration: Value,
    parameter: Value,
    context: &Context,
    append_log: &AppendLog,
) -> Result<(), String> {
    let configuration = Config::try_form_value(configuration)?;
    append_log(LogLevel::Info, String::from("正在解析参数"));
    let parameter = Parameter::try_form_value(parameter)?;
    append_log(LogLevel::Info, String::from("正在获取文件"));
    let file = context.download_file(&parameter.file.key).await?;
    append_log(LogLevel::Info, String::from("获取文件完成，开始部署"));
    let result = await_future(try_handle(file, configuration, append_log.clone())).await?;
    result?;
    append_log(LogLevel::Info, String::from("部署成功!"));
    return Ok(());
}

async fn try_handle(
    package: std::fs::File,
    configuration: Config,
    append_log: AppendLog,
) -> Result<(), String> {
    let package = parse_package(package)?;
    append_log(LogLevel::Info, String::from("开始获取上传token"));
    let token = fetch_upload_token(&configuration).await?;
    let token = Arc::new(Mutex::new(token));
    append_log(LogLevel::Info, String::from("开始上传文件"));
    let append_log_clone = append_log.clone();
    {
        let token = token.clone();
        try_max_times(
            async |_last_error| {
                let mut token = token.lock().await;
                match try_upload_files(&token, &package.static_files).await {
                    Ok(data) => Ok(data),
                    Err(error) => {
                        match &error {
                            UploadError::Unauthorized => {
                                match fetch_upload_token(&configuration).await {
                                    Ok(new_token) => {
                                        *token = new_token;
                                    }
                                    Err(error) => {
                                        append_log_clone(LogLevel::Warn, error);
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                    }
                                };
                            }
                            _ => {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                        Err(error)
                    }
                }
            },
            NonZeroU32::new(MAX_UPLOAD_ATTEMPTS).unwrap(),
        )
        .await
        .map_err(|err| err.to_string())?;
    }
    append_log(LogLevel::Info, String::from("开始更新文件哈希"));
    let mut token = token.lock().await;
    if let Err(error) = try_upsert_hashes(&token, &package.static_files).await {
        match error {
            UploadError::Unauthorized => {
                *token = fetch_upload_token(&configuration).await?;
                try_upsert_hashes(&token, &package.static_files)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            UploadError::CommonError(error) => {
                return Err(error);
            }
        }
    }
    drop(token);
    append_log(LogLevel::Info, String::from("开始提交部署"));
    let deploy_result = try_max_times(
        async |_last_error| match try_deploy(&configuration, &package).await {
            Ok(_data) => Ok(Ok(())),
            Err(error) => match error {
                DeployError::UnknownError => {
                    //UnknownError的错误对try_max_times才算错误，允许重试
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    Err(error)
                }
                DeployError::CommonError(error) => {
                    //其它的错误对try_max_times算成功，不允许重试，但实际上是失败
                    Ok(Err(error))
                }
            },
        },
        NonZeroU32::new(MAX_DEPLOYMENT_ATTEMPTS).unwrap(),
    )
    .await
    .map_err(|err| err.to_string())?;
    return deploy_result;
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiError {
    code: f64,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResult<T> {
    success: bool,
    errors: Option<Vec<ApiError>>,
    result: T,
}

impl<T> ApiResult<T> {
    fn is_error(&self, err_code: f64) -> bool {
        return self
            .errors
            .as_ref()
            .map(|errors| errors.iter().any(|error| err_code == error.code))
            .unwrap_or(false);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FileInfo {
    path: String,
    hash: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    contentType: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadPayloadFile {
    key: String,
    value: String,
    metadata: Metadata,
    base64: bool,
}

const SPECIAL_FILES: &[&str] = &["_worker.js", "_headers", "_redirects", "_routes.json"];

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    static_files: Vec<FileInfo>,
    special_files: HashMap<String, String>,
}

fn parse_package(package: std::fs::File) -> Result<Package, String> {
    let mut archive =
        zip::ZipArchive::new(package).map_err(|err| format!("文件包不是zip格式: {err}"))?;
    let file_count = archive.len();
    let mut files = Vec::with_capacity(file_count);
    let mut special_files: HashMap<String, String> = HashMap::new();
    for i in 0..file_count {
        let mut file = archive
            .by_index(i)
            .map_err(|err| format!("获取zip内部文件失败: {err}"))?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        let extension = outpath
            .extension()
            .map(|extension| extension.to_str())
            .flatten()
            .unwrap_or_default();
        let outpath = match outpath.to_str() {
            Some(outpath) => outpath.to_string(),
            None => continue,
        };
        let mut bytes = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut bytes)
            .map_err(|err| format!("读取zip内部文件失败: {err}"))?;
        if SPECIAL_FILES.contains(&outpath.as_str()) {
            let content = String::from_utf8(bytes)
                .map_err(|err| format!("文件{outpath}不是UTF-8格式文本: {err}"))?;
            special_files.insert(outpath, content);
            continue;
        }
        let value = BASE64_STANDARD.encode(&bytes);
        let mut hash = blake3::hash(format!("{}{}", value, extension).as_bytes())
            .to_hex()
            .as_str()
            .to_string();
        let _ = hash.split_off(32);
        let upload_payload_file = FileInfo {
            path: outpath,
            hash: hash,
            value: value,
        };
        files.push(upload_payload_file);
    }
    return Ok(Package {
        static_files: files,
        special_files: special_files,
    });
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadToken {
    jwt: String,
}

async fn fetch_upload_token(configuration: &Config) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/accounts/{}/pages/projects/{}/upload-token",
        CLOUDFLARE_API_BASE_URL, configuration.account_id, configuration.project_name
    );
    let url = Url::parse(&url).map_err(|err| format!("构造获取token的api地址失败: {err}"))?;
    let mut request = Request::new(Method::GET, url);
    request.headers_mut().typed_insert(
        Authorization::bearer(&configuration.api_token)
            .map_err(|err| format!("构造获取token的请求失败: {err}"))?,
    );
    let resp_text = RequestBuilder::from_parts(client, request)
        .send()
        .await
        .map_err(|err| format!("获取token失败: {err}"))?
        .text()
        .await
        .map_err(|err| format!("获取token失败: {err}"))?;
    let resp: ApiResult<UploadToken> = serde_json::from_str(&resp_text)
        .map_err(|_err| format!("获取token接口响应格式不正确: {resp_text}"))?;
    if resp.success {
        return Ok(resp.result.jwt);
    } else {
        return Err(format!("获取token失败, 接口响应：{}", resp_text));
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct HashesForm {
    hashes: Vec<String>,
}

async fn get_missing_hashes(token: &str, files: &[FileInfo]) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let url = Url::parse(&format!(
        "{}/pages/assets/check-missing",
        CLOUDFLARE_API_BASE_URL
    ))
    .map_err(|err| format!("构造检测遗漏hash的地址失败: {err}"))?;
    let mut request = Request::new(Method::POST, url);
    request.headers_mut().typed_insert(
        Authorization::bearer(&token).map_err(|err| format!("构造检测遗漏hash请求失败: {err}"))?,
    );
    let hashes: Vec<String> = files.iter().map(|file| file.hash.clone()).collect();
    let hashes_form = HashesForm { hashes };
    let resp_text = RequestBuilder::from_parts(client, request)
        .json(&hashes_form)
        .send()
        .await
        .map_err(|err| format!("检测遗漏hash失败: {err}"))?
        .text()
        .await
        .map_err(|err| format!("检测遗漏hash失败: {err}"))?;
    let resp: ApiResult<Vec<String>> = serde_json::from_str(&resp_text)
        .map_err(|_err| format!("检测遗漏hash接口响应格式不正确: {resp_text}"))?;
    if resp.success {
        return Ok(resp.result);
    } else {
        return Err(format!("检测遗漏hash失败, 接口响应：{}", resp_text));
    }
}

enum UploadError {
    CommonError(String),
    Unauthorized,
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UploadError::CommonError(error) => {
                write!(f, "{}", error)
            }
            UploadError::Unauthorized => {
                write!(f, "没有权限!")
            }
        }
    }
}

async fn try_upload_files(token: &str, files: &[FileInfo]) -> Result<(), UploadError> {
    let missing_hashes = get_missing_hashes(token, files)
        .await
        .map_err(UploadError::CommonError)?;
    if missing_hashes.is_empty() {
        return Ok(());
    }
    let files: Vec<UploadPayloadFile> = files
        .iter()
        .filter(|file| return missing_hashes.contains(&file.hash))
        .map(|file| {
            let mime = mime_guess::from_path(&file.path)
                .first_or_octet_stream()
                .to_string();
            UploadPayloadFile {
                key: file.hash.clone(),
                value: file.value.clone(),
                metadata: Metadata { contentType: mime },
                base64: true,
            }
        })
        .collect();
    let client = reqwest::Client::new();
    let url = Url::parse(&format!("{}/pages/assets/upload", CLOUDFLARE_API_BASE_URL))
        .map_err(|err| UploadError::CommonError(format!("构造上传地址失败: {err}")))?;
    let mut request = Request::new(Method::POST, url);
    request.headers_mut().typed_insert(
        Authorization::bearer(&token)
            .map_err(|err| UploadError::CommonError(format!("构造上传请求失败: {err}")))?,
    );
    let resp_text = RequestBuilder::from_parts(client, request)
        .json(&files)
        .send()
        .await
        .map_err(|err| UploadError::CommonError(format!("上传文件失败: {err}")))?
        .text()
        .await
        .map_err(|err| UploadError::CommonError(format!("上传文件失败: {err}")))?;
    let resp: ApiResult<Option<serde_json::Value>> = serde_json::from_str(&resp_text)
        .map_err(|_err| UploadError::CommonError(format!("上传文件响应格式不正确: {resp_text}")))?;
    if resp.success {
        return Ok(());
    } else {
        if resp.is_error(UNAUTHORIZED) {
            return Err(UploadError::Unauthorized);
        } else {
            return Err(UploadError::CommonError(format!(
                "上传文件失败, 接口响应：{}",
                resp_text
            )));
        }
    }
}

async fn try_upsert_hashes(token: &str, files: &[FileInfo]) -> Result<(), UploadError> {
    let client = reqwest::Client::new();
    let url = Url::parse(&format!(
        "{}/pages/assets/upsert-hashes",
        CLOUDFLARE_API_BASE_URL
    ))
    .map_err(|err| UploadError::CommonError(format!("构造更新文件哈希接口地址失败: {err}")))?;
    let mut request = Request::new(Method::POST, url);
    request.headers_mut().typed_insert(
        Authorization::bearer(&token)
            .map_err(|err| UploadError::CommonError(format!("构造请求失败: {err}")))?,
    );
    let hashes: Vec<String> = files.iter().map(|file| file.hash.clone()).collect();
    let hashes_form = HashesForm { hashes };
    let resp_text = RequestBuilder::from_parts(client, request)
        .json(&hashes_form)
        .send()
        .await
        .map_err(|err| UploadError::CommonError(format!("更新文件哈希失败: {err}")))?
        .text()
        .await
        .map_err(|err| UploadError::CommonError(format!("更新文件哈希失败: {err}")))?;
    let resp: ApiResult<Option<serde_json::Value>> =
        serde_json::from_str(&resp_text).map_err(|_err| {
            UploadError::CommonError(format!("更新文件哈希接口响应格式不正确: {resp_text}"))
        })?;
    if resp.success {
        return Ok(());
    } else {
        if resp.is_error(UNAUTHORIZED) {
            return Err(UploadError::Unauthorized);
        } else {
            return Err(UploadError::CommonError(format!(
                "更新文件哈希失败, 接口响应：{}",
                resp_text
            )));
        }
    }
}

enum DeployError {
    CommonError(String),
    UnknownError,
}

impl fmt::Display for DeployError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeployError::CommonError(error) => {
                write!(f, "{}", error)
            }
            DeployError::UnknownError => {
                write!(f, "未知错误!")
            }
        }
    }
}

async fn try_deploy(configuration: &Config, package: &Package) -> Result<(), DeployError> {
    let mut form = reqwest::multipart::Form::new();
    let manifest: HashMap<String, String> = package
        .static_files
        .iter()
        .map(|file| {
            return (format!("/{}", file.path), file.hash.clone());
        })
        .collect();
    let manifest = serde_json::to_string(&manifest)
        .map_err(|err| DeployError::CommonError(format!("构造文件清单失败: {err}")))?;
    form = form.text(String::from("manifest"), manifest);
    for (file_name, content) in package.special_files.iter() {
        if "_worker.js" == file_name {
            // _worker.js的处理太复杂，不支持
            continue;
        }
        let ext = Path::extension(file_name.as_ref())
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let mime = mime_guess::from_ext(ext).first_or_octet_stream();
        form = form.part(
            file_name.clone(),
            Part::bytes(content.clone().into_bytes())
                //必须设置文件名字，否则部署后不生效
                .file_name(file_name.clone())
                .mime_str(&mime.to_string())
                .map_err(|err| DeployError::CommonError(err.to_string()))?,
        );
    }
    let url = format!(
        "{}/accounts/{}/pages/projects/{}/deployments",
        CLOUDFLARE_API_BASE_URL, configuration.account_id, configuration.project_name
    );
    let client = reqwest::Client::new();
    let url = Url::parse(&url)
        .map_err(|err| DeployError::CommonError(format!("构造部署地址失败: {err}")))?;
    let mut request = Request::new(Method::POST, url);
    request.headers_mut().typed_insert(
        Authorization::bearer(&configuration.api_token)
            .map_err(|err| DeployError::CommonError(format!("构造部署请求失败: {err}")))?,
    );
    let resp_text = RequestBuilder::from_parts(client, request)
        .multipart(form)
        .send()
        .await
        .map_err(|err| DeployError::CommonError(format!("部署失败: {err}")))?
        .text()
        .await
        .map_err(|err| DeployError::CommonError(format!("部署失败: {err}")))?;
    let resp: ApiResult<Option<serde_json::Value>> = serde_json::from_str(&resp_text)
        .map_err(|_err| DeployError::CommonError(format!("部署接口响应格式不正确: {resp_text}")))?;
    if resp.success {
        return Ok(());
    } else {
        if resp.is_error(UNKNOWN_ERROR) {
            return Err(DeployError::UnknownError);
        } else {
            return Err(DeployError::CommonError(format!(
                "部署失败, 接口响应：{}",
                resp_text
            )));
        }
    }
}
