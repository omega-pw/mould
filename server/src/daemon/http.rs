use crate::action;
use crate::context::Context;
use crate::native_common;
use crate::sdk;
use crate::Asset;
use action::system::get_system_info::get_system_info;
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
use native_common::errno::result_to_json_resp;
use native_common::utils::HexStr;
use oauth2::{CsrfToken, PkceCodeChallenge};
use rust_embed::RustEmbed;
use sdk::system::get_system_info::GET_SYSTEM_INFO_API;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Handler;
use tihu_native::http::Body;
use tihu_native::http::HttpHandler;
use tihu_native::http::RequestData;
use tokio::net::TcpListener;

fn json_response<T: Into<Body>>(body: T) -> Response<Body> {
    let mut response = Response::new(body.into());
    response.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_str("application/json; charset=utf-8").unwrap(),
    );
    return response;
}

fn response_redirect(url: &str) -> Response<Body> {
    match HeaderValue::from_str(url) {
        Err(err) => {
            let mut response = text_response(err.to_string());
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

fn response_bad_request() -> Response<Body> {
    let status_code = StatusCode::BAD_REQUEST;
    let status_text = status_code.canonical_reason().unwrap_or("Bad Request");
    let mut response = Response::new(Body::from(status_text));
    *response.status_mut() = status_code;
    response.headers_mut().typed_insert(ContentType::html());
    return response;
}

fn response_html(content: String) -> Response<Body> {
    let mut response = Response::new(Body::from(content));
    response.headers_mut().typed_insert(ContentType::html());
    return response;
}

fn text_response<T: Into<Body>>(body: T) -> Response<Body> {
    let mut response = Response::new(body.into());
    response
        .headers_mut()
        .typed_insert(ContentType::text_utf8());
    return response;
}

fn response_not_found() -> Response<Body> {
    let status_code = StatusCode::NOT_FOUND;
    let status_text = status_code.canonical_reason().unwrap_or("Not Found");
    let mut response = Response::new(Body::from(status_text));
    *response.status_mut() = status_code;
    response
        .headers_mut()
        .typed_insert(ContentType::text_utf8());
    return response;
}

fn handle_embed<B: RustEmbed>(req: Request<Incoming>) -> Response<Body> {
    let mut path = req
        .uri()
        .path()
        .trim_start_matches('/')
        .trim_end_matches('/');
    let index_page = "index.html";
    if path.is_empty() {
        path = index_page;
    }
    let (content, actual_path) = B::get(path)
        .map(|content| (Some(content), path))
        .unwrap_or_else(|| (B::get(index_page), index_page));
    match content {
        Some(content) => {
            let hash = HexStr(&content.metadata.sha256_hash()).to_string();
            if req
                .headers()
                .get(header::IF_NONE_MATCH)
                .map(|etag| etag.to_str().unwrap_or("000000").eq(&hash))
                .unwrap_or(false)
            {
                let mut response = Response::new(Body::empty());
                *response.status_mut() = StatusCode::NOT_MODIFIED;
                return response;
            }
            let body = content.data.clone();
            let mime = mime_guess::from_path(actual_path).first_or_octet_stream();
            return Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::ETAG, hash)
                .body(Body::from(body))
                .unwrap();
        }
        None => {
            return response_not_found();
        }
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
    handler: Arc<
        impl Handler<
            (Request<Incoming>, SocketAddr, RequestData),
            Out = Result<Response<Body>, anyhow::Error>,
        >,
    >,
) -> Result<Response<Body>, anyhow::Error> {
    let route = req.uri().path();
    let oss_handler = context.get_oss_handler();
    if Method::GET == req.method() {
        if route.starts_with("/oauth2/login/") {
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
                    let mut response = text_response("Bad Request");
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
                    let mut response = text_response("Bad Request");
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
        if GET_SYSTEM_INFO_API == route {
            return Ok(json_response(result_to_json_resp(get_system_info().await)));
        }
        return handler.handle((req, remote_addr, RequestData::new())).await;
    } else {
        return Ok(response_not_found());
    }
}

async fn dispatch(
    context: Arc<Context>,
    req: Request<Incoming>,
    remote_addr: SocketAddr,
    handler: Arc<
        impl Handler<
            (Request<Incoming>, SocketAddr, RequestData),
            Out = Result<Response<Body>, anyhow::Error>,
        >,
    >,
) -> Result<Response<Body>, hyper::Error> {
    match try_dispatch(context, req, remote_addr, handler).await {
        Ok(response) => {
            return Ok(response);
        }
        Err(err) => {
            log::error!("处理请求失败: {}", err.to_string());
            let mut response = text_response("Internal Server Error");
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
    }
}

pub async fn start_service(
    context: Arc<Context>,
    handler: Arc<
        impl Handler<
            (Request<Incoming>, SocketAddr, RequestData),
            Out = Result<Response<Body>, anyhow::Error>,
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
        let handler = handler.clone();
        tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let context = context.clone();
                        let handler = handler.clone();
                        dispatch(context, req, remote_addr, handler)
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
