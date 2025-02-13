#![allow(non_snake_case)]
pub mod parameter;
use super::await_future;
use crate::config::Config;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use headers::{Authorization, HeaderMapExt};
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Context;
use mould_extension_sdk::LogLevel;
use parameter::Parameter;
use reqwest::Method;
use reqwest::Request;
use reqwest::RequestBuilder;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

static CLOUDFLARE_API_BASE_URL: &str = "https://api.cloudflare.com/client/v4";

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
    _append_log: AppendLog,
) -> Result<(), String> {
    let files = parse_package(package)?;
    let token = fetch_upload_token(&configuration).await?;
    upload_files(&token, &files).await?;
    upsert_hashes(&token, &files).await?;
    deploy(&configuration, &files).await?;
    return Ok(());
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResult<T> {
    success: bool,
    result: T,
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

fn parse_package(package: std::fs::File) -> Result<Vec<FileInfo>, String> {
    let mut archive =
        zip::ZipArchive::new(package).map_err(|err| format!("文件包不是zip格式: {err}"))?;
    let file_count = archive.len();
    let mut files = Vec::with_capacity(file_count);
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
        let value = BASE64_STANDARD.encode(&bytes);
        let hash = blake3::hash(format!("{}{}", value, extension).as_bytes())
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
    return Ok(files);
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
    let resp = RequestBuilder::from_parts(client, request)
        .send()
        .await
        .map_err(|err| format!("获取token失败: {err}"))?
        .json::<ApiResult<UploadToken>>()
        .await
        .map_err(|err| format!("获取token失败: {err}"))?;
    if resp.success {
        return Ok(resp.result.jwt);
    } else {
        return Err(String::from("获取token失败"));
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
    let resp = RequestBuilder::from_parts(client, request)
        .json(&hashes_form)
        .send()
        .await
        .map_err(|err| format!("检测遗漏hash失败: {err}"))?
        .json::<ApiResult<Vec<String>>>()
        .await
        .map_err(|err| format!("检测遗漏hash失败: {err}"))?;
    if resp.success {
        return Ok(resp.result);
    } else {
        return Err(String::from("检测遗漏hash失败"));
    }
}

async fn upload_files(token: &str, files: &[FileInfo]) -> Result<(), String> {
    let missing_hashes = get_missing_hashes(token, files).await?;
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
        .map_err(|err| format!("构造上传地址失败: {err}"))?;
    let mut request = Request::new(Method::POST, url);
    request.headers_mut().typed_insert(
        Authorization::bearer(&token).map_err(|err| format!("构造上传请求失败: {err}"))?,
    );
    let resp = RequestBuilder::from_parts(client, request)
        .json(&files)
        .send()
        .await
        .map_err(|err| format!("上传文件失败: {err}"))?
        .json::<ApiResult<serde_json::Value>>()
        .await
        .map_err(|err| format!("上传文件失败: {err}"))?;
    if resp.success {
        return Ok(());
    } else {
        return Err(String::from("上传文件失败"));
    }
}

async fn upsert_hashes(token: &str, files: &[FileInfo]) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = Url::parse(&format!(
        "{}/pages/assets/upsert-hashes",
        CLOUDFLARE_API_BASE_URL
    ))
    .map_err(|err| format!("构造更新文件哈希接口地址失败: {err}"))?;
    let mut request = Request::new(Method::POST, url);
    request
        .headers_mut()
        .typed_insert(Authorization::bearer(&token).map_err(|err| format!("构造请求失败: {err}"))?);
    let hashes: Vec<String> = files.iter().map(|file| file.hash.clone()).collect();
    let hashes_form = HashesForm { hashes };
    let resp = RequestBuilder::from_parts(client, request)
        .json(&hashes_form)
        .send()
        .await
        .map_err(|err| format!("更新文件哈希失败: {err}"))?
        .json::<ApiResult<Option<()>>>()
        .await
        .map_err(|err| format!("更新文件哈希失败: {err}"))?;
    if resp.success {
        return Ok(());
    } else {
        return Err(String::from("更新文件哈希失败"));
    }
}

async fn deploy(configuration: &Config, files: &[FileInfo]) -> Result<(), String> {
    let mut form = reqwest::multipart::Form::new();
    let manifest: HashMap<String, String> = files
        .iter()
        .map(|file| {
            return (format!("/{}", file.path), file.hash.clone());
        })
        .collect();
    let manifest =
        serde_json::to_string(&manifest).map_err(|err| format!("构造文件清单失败: {err}"))?;
    form = form.text(String::from("manifest"), manifest);
    let url = format!(
        "{}/accounts/{}/pages/projects/{}/deployments",
        CLOUDFLARE_API_BASE_URL, configuration.account_id, configuration.project_name
    );
    let client = reqwest::Client::new();
    let url = Url::parse(&url).map_err(|err| format!("构造部署地址失败: {err}"))?;
    let mut request = Request::new(Method::POST, url);
    request.headers_mut().typed_insert(
        Authorization::bearer(&configuration.api_token)
            .map_err(|err| format!("构造部署请求失败: {err}"))?,
    );
    let resp = RequestBuilder::from_parts(client, request)
        .multipart(form)
        .send()
        .await
        .map_err(|err| format!("部署失败: {err}"))?
        .json::<ApiResult<serde_json::Value>>()
        .await
        .map_err(|err| format!("部署失败: {err}"))?;
    if resp.success {
        return Ok(());
    } else {
        return Err(String::from("部署失败"));
    }
}
