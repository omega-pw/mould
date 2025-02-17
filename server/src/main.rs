mod action;
mod config;
mod context;
mod daemon;
mod middleware;
mod model;
mod native_common;
mod route;
mod service;
use config::Arguments;
use config::Config;
use context::Context;
use headers::{ContentType, HeaderMapExt};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::header::HeaderValue;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
pub use log;
use middleware::auth::ApiLevel;
use middleware::auth::AuthMiddleware;
use middleware::auth::Guest;
use middleware::auth::User;
use middleware::context::ContextMiddleware;
use middleware::session::SessionMiddleware;
use middleware::session::SignatureResult;
use middleware::time_stat::TimeStatMiddleware;
use native_common::errno::json_serialize_err;
use native_common::middleware::CountStatMiddleware;
use native_common::utils::sha512;
use route::dispatch_guest_api;
use route::dispatch_user_api;
use route::WHITE_LIST_NAMESPACE;
use rust_embed::RustEmbed;
pub use server_sdk as sdk;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Handler;
use tihu::LightString;
use tihu::Middleware;
use tihu_native::http::Body;
use tihu_native::http::HttpHandler;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;

pub const VERSION_INFO: &'static str = include_str!("../version.txt");

#[derive(RustEmbed)]
#[folder = "./static/"]
struct Asset;

tokio::task_local! {
    pub static CONTEXT: Arc<Context>;
}

pub fn get_context() -> Result<Arc<Context>, ErrNo> {
    return CONTEXT
        .try_with(|context| context.clone())
        .map_err(|_err| ErrNo::CommonError(LightString::from_static("获取应用程序上下文失败！")));
}

// pub struct Request {
//     pub client_id: ClientId,
//     pub route: LightString,
//     pub session: Option<LightString>,
//     pub service_version: Option<HashMap<String, String>>,
//     pub context: Option<LightString>,
//     pub body: Bytes,
//     pub request_ip: Bytes,
// }

// pub struct Response {
//     pub session: Option<LightString>,
//     pub body: Bytes,
// }

// struct MultiHttpHandler {
//     http_handlers: Vec<Arc<dyn HttpHandler>>,
// }

fn match_route(http_handler: &dyn HttpHandler, route: &str) -> bool {
    return http_handler
        .namespace()
        .iter()
        .any(|namespace| route.starts_with(namespace.as_ref()));
}

// fn response_not_found() -> Response<Body> {
//     let status_code = StatusCode::NOT_FOUND;
//     let status_text = status_code.canonical_reason().unwrap_or("Not Found");
//     let mut response = Response::new(Body::from(status_text));
//     *response.status_mut() = status_code;
//     response
//         .headers_mut()
//         .typed_insert(ContentType::text_utf8());
//     return response;
// }

// #[async_trait]
// impl Handler<(Request<Incoming>, SocketAddr)> for MultiHttpHandler {
//     type Out = Result<Response<Body>, hyper::Error>;
//     async fn handle(&self, (req, remote_addr): (Request<Incoming>, SocketAddr)) -> Self::Out {
//         for http_handler in self.http_handlers.iter() {
//             if match_route(http_handler.as_ref(), req.uri().path()) {
//                 let resp = http_handler.handle(req, remote_addr, None).await?;
//                 return Ok(resp.map(From::from));
//             }
//         }
//         return Ok(response_not_found());
//     }
// }

fn text_response<T: Into<Body>>(body: T) -> Response<Body> {
    let mut response = Response::new(body.into());
    response
        .headers_mut()
        .typed_insert(ContentType::text_utf8());
    return response;
}

fn json_response<T: Into<Body>>(body: T) -> Response<Body> {
    let mut response = Response::new(body.into());
    response.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_str("application/json; charset=utf-8").unwrap(),
    );
    return response;
}

async fn dispatch_api(
    (request, remote_addr, mut request_data): (Request<Incoming>, SocketAddr, RequestData),
) -> Result<Response<Body>, anyhow::Error> {
    let signature_result = request_data
        .try_get::<SignatureResult>(&request, remote_addr)
        .await?;
    let body_hash = signature_result.body_hash.clone();
    let api_level = request_data
        .try_get::<ApiLevel>(&request, remote_addr)
        .await?
        .clone();

    let resp_ret = match api_level {
        ApiLevel::Guest => {
            let guest: Guest = request_data
                .try_get::<Guest>(&request, remote_addr)
                .await?
                .clone();
            let (parts, body) = request.into_parts();
            let route = parts.uri.path();
            let body = body.collect().await?.to_bytes();
            let actual_hash = sha512(&body);
            if actual_hash.as_slice() != &body_hash {
                log::error!("请求体hash不一致: {}", route);
                let mut response = text_response("Bad Request");
                *response.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(response);
            }
            dispatch_guest_api((route.to_string().into(), body, guest)).await
        }
        ApiLevel::User => {
            let user: User = request_data
                .try_get::<User>(&request, remote_addr)
                .await?
                .clone();
            let (parts, body) = request.into_parts();
            let route = parts.uri.path();
            let body = body.collect().await?.to_bytes();
            let actual_hash = sha512(&body);
            if actual_hash.as_slice() != &body_hash {
                log::error!("请求体hash不一致: {}", route);
                let mut response = text_response("Bad Request");
                *response.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(response);
            }
            dispatch_user_api((route.to_string().into(), body, user)).await
        }
    };
    match resp_ret {
        Ok(resp) => {
            return Ok(json_response(resp));
        }
        Err(err_msg) => {
            let resp = tihu::api::Response::<()>::from(err_msg);
            let resp = serde_json::to_vec(&resp).unwrap_or_else(json_serialize_err);
            return Ok(json_response(resp));
        }
    }
}

pub async fn get_handler(
    context: Arc<Context>,
) -> Result<
    impl Handler<
        (Request<Incoming>, SocketAddr, RequestData),
        Out = Result<Response<Body>, anyhow::Error>,
    >,
    anyhow::Error,
> {
    let oss_handler = context.get_oss_handler().clone();
    let api_handler = Arc::new(TimeStatMiddleware::new().transform(dispatch_api));
    let white_list_namespace = WHITE_LIST_NAMESPACE
        .iter()
        .map(|namespace| LightString::from_static(*namespace))
        .collect::<Vec<_>>();
    let handler = ContextMiddleware::new(context.clone())
        .chain(CountStatMiddleware::new())
        .chain(SessionMiddleware::new(context.clone()))
        .chain(AuthMiddleware::new(context.clone(), white_list_namespace))
        .transform(
            move |(request, remote_addr, mut request_data): (
                Request<Incoming>,
                SocketAddr,
                RequestData,
            )| {
                let oss_handler = oss_handler.clone();
                let api_handler = api_handler.clone();
                async move {
                    let result = if match_route(oss_handler.as_ref(), request.uri().path()) {
                        let resp = oss_handler
                            .handle(request, remote_addr, &mut request_data, None)
                            .await;
                        resp.map(|resp| resp.map(Body::from))
                    } else {
                        api_handler
                            .handle((request, remote_addr, request_data))
                            .await
                    };
                    result
                }
            },
        );
    let handler = Arc::new(handler);
    return Ok(
        move |(req, remote_addr, request_data): (Request<Incoming>, SocketAddr, RequestData)| {
            let handler = handler.clone();
            async move { handler.handle((req, remote_addr, request_data)).await }
        },
    );
}

fn init_v8() {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    init_v8();
    let config = Arguments::try_from_args()?;
    let config = Config::try_load_from_file(&config.config_path)?;
    let context = Context::try_init_from_config(config).await?;
    let context = Arc::new(context);
    let handler = get_handler(context.clone()).await?;
    let handler = Arc::new(handler);
    daemon::http::start_service(context, handler).await?;
    Ok(())
}
