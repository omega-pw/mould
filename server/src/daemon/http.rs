use crate::action;
use crate::context::Context;
use crate::native_common;
use crate::sdk;
use crate::Asset;
use action::system::get_current_time::get_current_time;
use form_urlencoded::Serializer;
use headers::{ContentType, HeaderMapExt};
use hyper::body::Incoming;
use hyper::header;
use hyper::header::HeaderValue;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::tokio::TokioIo;
use hyper_util::rt::TokioExecutor;
use hyper_util::server::conn::auto;
use mime_guess::Mime;
use native_common::errno::json_serialize_err;
use native_common::errno::result_to_json_resp;
use native_common::utils::http::response_html;
use native_common::utils::http::response_json;
use native_common::utils::http::response_not_found;
use native_common::utils::http::response_text;
use native_common::utils::HexStr;
use oauth2::{CsrfToken, PkceCodeChallenge};
use rust_embed::EmbeddedFile;
use rust_embed::RustEmbed;
use sdk::system::get_current_time::GET_CURRENT_TIME_API;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Handler;
use tihu_native::http::Body;
use tihu_native::http::HttpHandler;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;
use tokio::net::TcpListener;

fn response_redirect(url: &str) -> Response<Body> {
    match HeaderValue::from_str(url) {
        Err(err) => {
            let mut response = response_text(err.to_string());
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return response;
        }
        Ok(location) => {
            let status_code = StatusCode::FOUND;
            let status_text = status_code.canonical_reason().unwrap_or("Found");
            let mut response = Response::new(Body::from(status_text));
            *response.status_mut() = status_code;
            response.headers_mut().typed_insert(ContentType::html());
            response.headers_mut().insert(header::LOCATION, location);
            return response;
        }
    }
}

struct ReadResult {
    content: EmbeddedFile,
    content_type: Mime,
    content_encoding: Option<&'static str>,
}

fn read_by_file<B: RustEmbed>(path: &str, accepts_gzip: bool) -> Option<ReadResult> {
    let mut read_result = None;
    if accepts_gzip {
        let gz_path = format!("{}.gz", path);
        if let Some(content) = B::get(&gz_path) {
            read_result.replace(ReadResult {
                content: content,
                content_type: mime_guess::from_path(path).first_or_octet_stream(),
                content_encoding: Some("gzip"),
            });
        }
    }
    if read_result.is_none() {
        if let Some(content) = B::get(path) {
            read_result.replace(ReadResult {
                content: content,
                content_type: mime_guess::from_path(path).first_or_octet_stream(),
                content_encoding: None,
            });
        }
    }
    return read_result;
}

fn handle_embed<B: RustEmbed>(req: Request<Incoming>) -> Response<Body> {
    let accepts_gzip = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase().contains("gzip"))
        .unwrap_or(false);
    let mut path = req
        .uri()
        .path()
        .trim_start_matches('/')
        .trim_end_matches('/');
    let index_page = "index.html";
    if path.is_empty() {
        path = index_page;
    }
    let mut cache_control = "public, must-revalidate, max-age=300";
    if ["index.html", "index.js", "index.css"]
        .iter()
        .any(|item| &path == item)
    {
        cache_control = "public, no-cache";
    }
    let mut read_result = read_by_file::<B>(path, accepts_gzip);
    if read_result.is_none() {
        read_result = read_by_file::<B>(index_page, accepts_gzip);
        // 单页应用回退按主页的逻辑处理
        cache_control = "public, no-cache";
    }
    if let Some(read_result) = read_result {
        let etag = format!(
            "\"{}\"",
            HexStr(&read_result.content.metadata.sha256_hash())
        );
        if req
            .headers()
            .get(header::IF_NONE_MATCH)
            .map(|req_etag| req_etag.to_str().ok() == Some(&etag))
            .unwrap_or(false)
        {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_MODIFIED;
            return response;
        }
        let body = read_result.content.data.clone();
        let mut builder = Response::builder();
        if let Some(content_encoding) = read_result.content_encoding {
            builder = builder.header(header::CONTENT_ENCODING, content_encoding);
        }
        return builder
            .header(header::CONTENT_TYPE, read_result.content_type.as_ref())
            .header(header::CONTENT_LENGTH, body.len())
            .header(header::CACHE_CONTROL, cache_control)
            .header(header::ETAG, etag)
            .body(Body::from(body))
            .unwrap();
    } else {
        return response_not_found();
    }
}

fn match_route(http_handler: &dyn HttpHandler, route: &str) -> bool {
    return http_handler
        .namespace()
        .iter()
        .any(|namespace| route.starts_with(namespace.as_ref()));
}

fn gen_login_html(
    auth_url: &str,
    provider: &str,
    csrf_token: &str,
    pkce_verifier: Option<&str>,
) -> String {
    let pkce_verifier_script = if let Some(pkce_verifier) = pkce_verifier {
        format!(r#"localStorage.setItem("pkce_verifier_{provider}", "{pkce_verifier}");"#)
    } else {
        format!(r#"localStorage.removeItem("pkce_verifier_{provider}");"#)
    };
    return format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>mould</title>
</head>
<body>
    <script type="text/javascript">
        localStorage.setItem("csrf_token_{provider}", "{csrf_token}");
        {pkce_verifier_script}
        window.location = "{auth_url}";
    </script>
</body>
</html>"#
    );
}

async fn try_dispatch(
    context: Arc<Context>,
    req: Request<Incoming>,
    remote_addr: SocketAddr,
    api_handler: Arc<
        impl Handler<
            (Arc<Context>, Request<Incoming>, SocketAddr, RequestData),
            Out = Result<Response<Body>, ErrNo>,
        >,
    >,
) -> Result<Response<Body>, anyhow::Error> {
    let route = req.uri().path();
    let oss_handler = context.get_oss_handler();
    if Method::GET == req.method() {
        if "/version.txt" == route {
            return Ok(response_text(crate::VERSION_INFO));
        } else if route.starts_with("/oauth2/login/") {
            let (_, provider) = route.split_at("/oauth2/login/".len());
            match context.get_oauth2_client(provider) {
                Ok((oauth2_client, oauth2_server)) => {
                    if "wechat" == provider {
                        let csrf_token = CsrfToken::new_random();
                        let csrf_token = csrf_token.secret();
                        let auth_url = &oauth2_server.auth_url;
                        let app_id = &oauth2_server.client_id;
                        let redirect_uri = format!(
                            "{}/oauth2/authorize/{}",
                            context.config.public_path, provider
                        );
                        let query: String = Serializer::new(String::new())
                            .append_pair("appid", app_id)
                            .append_pair("redirect_uri", &redirect_uri)
                            .append_pair("response_type", "code")
                            .append_pair("scope", "snsapi_userinfo")
                            .append_pair("state", csrf_token)
                            .finish();
                        let auth_url = format!("{auth_url}?{query}#wechat_redirect");
                        let response =
                            response_html(gen_login_html(&auth_url, provider, csrf_token, None));
                        return Ok(response);
                    } else {
                        let mut client = oauth2_client.authorize_url(CsrfToken::new_random);
                        if let Some(scopes) = oauth2_server.scopes.as_ref() {
                            client = client.add_scopes(scopes.clone());
                        }
                        let mut pkce_verifier_opt = None;
                        if oauth2_server.pkce {
                            let (pkce_challenge, pkce_verifier) =
                                PkceCodeChallenge::new_random_sha256();
                            client = client.set_pkce_challenge(pkce_challenge);
                            pkce_verifier_opt.replace(pkce_verifier.secret().clone());
                        }
                        let (auth_url, csrf_token) = client.url();
                        let response = response_html(gen_login_html(
                            &auth_url.to_string(),
                            provider,
                            csrf_token.secret(),
                            pkce_verifier_opt.as_deref(),
                        ));
                        return Ok(response);
                    }
                }
                Err(err_no) => {
                    log::error!("没有获取到对应的oauth2 provider: {}", err_no.to_string());
                    let mut response = response_text("Bad Request");
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            }
        } else if route.starts_with("/oidc/login/") {
            let (_, provider) = route.split_at("/oidc/login/".len());
            match context.get_openid_client(provider) {
                Ok((openid_client, openid_server)) => {
                    let scopes: Option<String> =
                        openid_server.scopes.as_ref().map(|scopes| scopes.join(" "));
                    let redirect_url = openid_client.auth_uri(scopes.as_deref(), None);
                    let response = response_redirect(&redirect_url.to_string());
                    return Ok(response);
                }
                Err(err_no) => {
                    log::error!("没有获取到对应的oidc provider: {}", err_no.to_string());
                    let mut response = response_text("Bad Request");
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            }
        } else if match_route(oss_handler.as_ref(), route) {
            let mut request_data = RequestData::new();
            let resp = oss_handler
                .handle(req, remote_addr, &mut request_data, None)
                .await?;
            return Ok(resp.map(From::from));
        } else {
            return Ok(handle_embed::<Asset>(req));
        }
    } else if Method::POST == req.method() {
        let route = route.to_string();
        if GET_CURRENT_TIME_API == route {
            return Ok(response_json(result_to_json_resp(get_current_time().await)));
        }
        if match_route(oss_handler.as_ref(), &route) {
            let mut request_data = RequestData::new();
            let resp = oss_handler
                .handle(req, remote_addr, &mut request_data, None)
                .await?;
            return Ok(resp.map(From::from));
        } else {
            match api_handler
                .handle((context, req, remote_addr, RequestData::new()))
                .await
            {
                Ok(resp) => {
                    return Ok(resp);
                }
                Err(err_msg) => {
                    let resp = tihu::api::Response::<()>::from(err_msg);
                    let resp = serde_json::to_vec(&resp).unwrap_or_else(json_serialize_err);
                    return Ok(response_json(resp));
                }
            }
        }
    } else {
        return Ok(response_not_found());
    }
}

async fn dispatch(
    context: Arc<Context>,
    req: Request<Incoming>,
    remote_addr: SocketAddr,
    api_handler: Arc<
        impl Handler<
            (Arc<Context>, Request<Incoming>, SocketAddr, RequestData),
            Out = Result<Response<Body>, ErrNo>,
        >,
    >,
) -> Result<Response<Body>, hyper::Error> {
    match try_dispatch(context, req, remote_addr, api_handler).await {
        Ok(response) => {
            return Ok(response);
        }
        Err(err) => {
            log::error!("处理请求失败: {}", err.to_string());
            let mut response = response_text("Internal Server Error");
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
    }
}

pub async fn start_service(
    context: Arc<Context>,
    api_handler: Arc<
        impl Handler<
            (Arc<Context>, Request<Incoming>, SocketAddr, RequestData),
            Out = Result<Response<Body>, ErrNo>,
        >,
    >,
) -> Result<(), anyhow::Error> {
    let bind_addr = SocketAddr::new(context.config.host, context.config.port);
    let listener = TcpListener::bind(bind_addr).await?;
    let actual_addr = listener.local_addr()?;
    log::info!("Listening on http://{}", actual_addr);
    println!("Listening on http://{}", actual_addr);
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let context = context.clone();
        let api_handler = api_handler.clone();
        tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let context = context.clone();
                        let api_handler = api_handler.clone();
                        dispatch(context, req, remote_addr, api_handler)
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
