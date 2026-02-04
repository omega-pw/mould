use crate::sdk;
use crate::utils::request::ApiExt;
use crate::utils::script_loader::ScriptLoader;
use crate::SharedString;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use sdk::system::get_system_info::GetSystemInfoApi;
use sdk::system::get_system_info::GetSystemInfoReq;
use std::sync::LazyLock;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

static TURNSTILE_LOADER: LazyLock<ScriptLoader> = LazyLock::new(|| {
    ScriptLoader::new(
        "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit",
        "turnstile",
    )
});

#[derive(PartialEq, Clone)]
pub enum TokenResult {
    NotRequired,
    Success(SharedString),
    Failure(SharedString),
}

#[component]
pub fn Turnstile(
    #[prop(into, optional)] style: SharedString,
    #[prop(into)] ondone: UnsyncCallback<TokenResult>,
) -> impl IntoView {
    let site_key: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let div_ref: NodeRef<html::Div> = NodeRef::new();
    if !cfg!(feature = "ssr") {
        spawn_local({
            let site_key = site_key.clone();
            let ondone = ondone.clone();
            async move {
                let result = get_site_key().await;
                match result {
                    Ok(key) => {
                        if let Some(key) = key {
                            site_key.set(Some(key.into()));
                        } else {
                            ondone.run(TokenResult::NotRequired);
                        }
                    }
                    Err(error) => {
                        ondone.run(TokenResult::Failure(error));
                    }
                }
            }
        });
        Effect::watch(
            move || (div_ref.get(), site_key.get()),
            move |(div_ref, site_key), _, _| {
                if let (Some(div), Some(site_key)) = (div_ref.clone(), site_key.clone()) {
                    spawn_local({
                        let ondone = ondone.clone();
                        async move {
                            match TURNSTILE_LOADER.get_or_load().await {
                                Ok(turnstile) => {
                                    let render_method: js_sys::Function = js_sys::Reflect::get(
                                        &turnstile,
                                        &JsValue::from_str("render"),
                                    )
                                    .unwrap()
                                    .dyn_into()
                                    .unwrap();
                                    let config = js_sys::Object::new();
                                    js_sys::Reflect::set(
                                        &config,
                                        &JsValue::from_str("appearance"),
                                        &JsValue::from_str("interaction-only"),
                                    )
                                    .unwrap();
                                    js_sys::Reflect::set(
                                        &config,
                                        &JsValue::from_str("sitekey"),
                                        &JsValue::from_str(&site_key),
                                    )
                                    .unwrap();
                                    let callback =
                                        Closure::wrap(Box::new(move |token: JsValue| -> () {
                                            if let Some(token) = token.as_string() {
                                                ondone.run(TokenResult::Success(
                                                    SharedString::from(token),
                                                ));
                                            } else {
                                                log::error!(
                                                "turnstile校验失败，token不是字符串，token: {:?}",
                                                token
                                            );
                                                ondone.run(TokenResult::Failure(
                                                    SharedString::from("环境检测未通过!"),
                                                ));
                                            }
                                        })
                                            as Box<dyn FnMut(JsValue) -> ()>)
                                        .into_js_value();
                                    js_sys::Reflect::set(
                                        &config,
                                        &JsValue::from_str("callback"),
                                        &callback,
                                    )
                                    .unwrap();
                                    let error_callback =
                                        Closure::wrap(Box::new(move |error_code: JsValue| -> () {
                                            log::error!(
                                                "turnstile校验失败，错误码: {:?}",
                                                error_code
                                            );
                                            ondone.run(TokenResult::Failure(SharedString::from(
                                                "环境检测未通过!",
                                            )));
                                        })
                                            as Box<dyn FnMut(JsValue) -> ()>)
                                        .into_js_value();
                                    js_sys::Reflect::set(
                                        &config,
                                        &JsValue::from_str("error-callback"),
                                        &error_callback,
                                    )
                                    .unwrap();
                                    let widget_id =
                                        render_method.call2(&turnstile, &div, &config).unwrap();
                                }
                                Err(error) => {
                                    log::error!("加载turnstile sdk失败: {:?}", error);
                                    ondone.run(TokenResult::Failure(SharedString::from(
                                        "环境检测失败!",
                                    )));
                                }
                            }
                        }
                    });
                }
            },
            false,
        );
    }
    move || {
        if site_key.get().is_some() {
            view! {
                <div node_ref={div_ref} style={style.clone()}/>
            }
            .into_any()
        } else {
            view! {}.into_any()
        }
    }
}

async fn get_site_key() -> Result<Option<String>, SharedString> {
    let params = GetSystemInfoReq {};
    let system_info = GetSystemInfoApi.call(&params).await?;
    return Ok(system_info.turnstile_site_key);
}
