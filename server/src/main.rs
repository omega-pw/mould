mod action;
mod config;
mod context;
mod daemon;
mod middleware;
mod model;
mod native_common;
mod prehandle;
mod route;
mod service;
use config::Arguments;
use config::Config;
use context::Context;
use hyper::body::Incoming;
use hyper::Request;
use hyper::Response;
pub use log;
use middleware::session::SessionMiddleware;
use native_common::middleware::CompressMiddleware;
use native_common::middleware::CountStatMiddleware;
use native_common::middleware::TimeStatMiddleware;
use native_common::utils::http::response_json;
use route::dispatch;
use rust_embed::RustEmbed;
pub use server_sdk as sdk;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Middleware;
use tihu::SharedString;
use tihu_native::http::Body;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;

pub const VERSION_INFO: &'static str = include_str!("../version.txt");

#[derive(RustEmbed)]
#[folder = "./static/"]
struct Asset;

async fn dispatch_api(
    (context, request, remote_addr, data_cache): (
        Arc<Context>,
        Request<Incoming>,
        SocketAddr,
        RequestData,
    ),
) -> Result<Response<Body>, ErrNo> {
    dispatch((context, request, remote_addr, data_cache))
        .await
        .map(response_json)
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
    let config = Config::try_load_from_file(&config.config_path).await?;
    let context = Context::try_init_from_config(config).await?;
    let context = Arc::new(context);
    let api_handler = CompressMiddleware::new()
        .chain(CountStatMiddleware::new())
        .chain(TimeStatMiddleware::new(None))
        .chain(SessionMiddleware::new(context.clone()))
        .transform(dispatch_api);
    let api_handler = Arc::new(api_handler);
    daemon::http::start_service(context, api_handler).await?;
    Ok(())
}
