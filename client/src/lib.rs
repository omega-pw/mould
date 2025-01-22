#![recursion_limit = "1024"]

mod app;
mod assets;
mod components;
mod fragment;
mod id_gen;
mod js;
mod logger;
mod pages;
mod route;
mod utils;

use id_gen::IdGen;
use sdk::auth::get_curr_user::User;
pub use server_sdk as sdk;
use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use tihu::api::Response;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

static mut ID_GEN: Option<IdGen> = None;

pub type LightString = yew::virtual_dom::AttrValue;

#[derive(Debug, Clone, PartialEq, Default, Properties)]
pub struct Context {
    pub curr_user: Option<User>,
}

pub enum ContextAction {
    UpdateUser(User),
}

impl Reducible for Context {
    type Action = ContextAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let app_context = match action {
            ContextAction::UpdateUser(curr_user) => Context {
                curr_user: Some(curr_user),
            },
        };
        app_context.into()
    }
}

pub type AppContext = UseReducerHandle<Context>;

pub struct ArcFn<In, Out>(Arc<dyn Fn(In) -> Out>);

impl<In, Out, F: Fn(In) -> Out + 'static> From<F> for ArcFn<In, Out> {
    fn from(func: F) -> Self {
        ArcFn(Arc::new(func))
    }
}

impl<In, Out> Clone for ArcFn<In, Out> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<In, Out> std::ops::Deref for ArcFn<In, Out> {
    type Target = dyn Fn(In) -> Out;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<In, Out> PartialEq for ArcFn<In, Out> {
    fn eq(&self, other: &ArcFn<In, Out>) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<In, Out> std::fmt::Debug for ArcFn<In, Out> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArcFn<_,_>")
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Lazyable<T> {
    Literal(T),
    Lazy(ArcFn<(), T>),
}

impl<T> Lazyable<T>
where
    T: ToOwned<Owned = T>,
{
    fn get<'a>(&'a self) -> Cow<'a, T> {
        match self {
            Lazyable::Literal(value) => Cow::Borrowed(value),
            Lazyable::Lazy(lazy_fn) => Cow::Owned(lazy_fn(())),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct NotifyCount {
    pub unread_feedback: u64,
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn init_request() {
    unsafe {
        utils::request::DEFAULT_HTTP_REQUESTOR =
            Some(Arc::new(
                |(url, req): (LightString, LightString)| -> Pin<
                    Box<dyn Future<Output = Result<LightString, LightString>>>,
                > { Box::pin(utils::default_http_requestor((url, req))) },
            ));
        utils::request::DEFAULT_LOADING_HANDLER = Some(Arc::new(
            |state: bool| -> Pin<Box<dyn Future<Output = ()>>> {
                if state {
                    components::loading::show();
                } else {
                    components::loading::hide();
                }
                Box::pin(async {})
            },
        ));
        utils::request::DEFAULT_DATA_UNWRAPPER = Some(Arc::new(
            |value: serde_json::Value| -> Pin<Box<dyn Future<Output = Result<serde_json::Value, LightString>>>> {
                let result = serde_json::from_value::<Response<serde_json::Value>>(value).map_err(|err| -> LightString {
                    log::error!("响应数据格式不正确：{}", err);
                    return LightString::from("响应数据格式不正确");
                });
                Box::pin(async {
                    match result {
                        Ok(resp) => {
                            if 0 == resp.code {
                                Ok(resp.data.unwrap_or(serde_json::Value::Null))
                            } else if -1 == resp.code {
                                let window = web_sys::window().unwrap();
                                window
                                    .alert_with_message("You are already logged in another place or login timeout, please log in again.")
                                    .unwrap();
                                window.location().assign("/login").unwrap();
                                Err(LightString::from(resp.message.to_string()))
                            } else {
                                Err(LightString::from(resp.message.to_string()))
                            }
                        },
                        Err(message) => {
                            Err(message)
                        }
                    }
                })
            },
        ));
        utils::request::DEFAULT_REQ_ERROR_HANDLER = Some(Arc::new(
            |message: LightString| -> Pin<Box<dyn Future<Output = ()>>> {
                components::popup_message::error(message);
                Box::pin(async {})
            },
        ));
        utils::request::DEFAULT_UNWRAP_ERROR_HANDLER = Some(Arc::new(
            |message: LightString| -> Pin<Box<dyn Future<Output = ()>>> {
                components::popup_message::error(message);
                Box::pin(async {})
            },
        ));
    }
}

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    unsafe {
        ID_GEN = Some(IdGen::new(None));
    }
    utils::set_panic_hook();
    logger::init();
    init_request();
    wasm_bindgen_futures::spawn_local(async move {
        utils::init_time_diff().await.ok();
        yew::Renderer::<app::App>::new().render();
    });
    Ok(())
}
