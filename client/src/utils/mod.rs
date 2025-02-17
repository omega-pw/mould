pub mod ajax;
pub mod binding;
pub mod handle;
pub mod list;
pub mod request;
pub mod validator;
use crate::components;
use crate::js;
use crate::sdk;
use crate::LightString;
use base64::engine::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::Utc;
pub use components::popup_message::error;
pub use components::popup_message::success;
pub use components::popup_message::warning;
use gloo::timers::callback::Timeout;
use js::is_valid_rsa_key_pair;
use js::sha512;
use js::RsaPriKey2048;
use js::RsaPubKey2048;
use js_sys::Function;
use js_sys::Math::random;
use js_sys::Promise;
use js_sys::Uint8Array;
use log;
use request::ApiExt;
use sdk::storage::UPLOAD_API;
use sdk::system::get_system_info::GetSystemInfoApi;
use sdk::system::get_system_info::GetSystemInfoReq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use tihu::api::Response as CommonResponse;
use tihu::client_id::ClientId;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FormData, Request, RequestInit, Response};
use yew::prelude::*;

static TIME_DIFF: AtomicI64 = AtomicI64::new(0);

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn format_time_local(date_time: &DateTime<Utc>) -> NaiveDateTime {
    return date_time.with_timezone(&Local).naive_local();
}

pub async fn init_time_diff() -> Result<(), LightString> {
    let client_time = Utc::now().timestamp_millis();
    let params = GetSystemInfoReq {};
    let system_info = GetSystemInfoApi.call(&params).await?;
    let server_time = system_info.current_time.timestamp_millis();
    let diff = server_time - client_time;
    TIME_DIFF.store(diff, Ordering::Relaxed);
    return Ok(());
}

pub fn get_server_time() -> DateTime<Utc> {
    return Utc::now() + Duration::milliseconds(TIME_DIFF.load(Ordering::Relaxed));
}

fn calc_client_id(api: &[u8], body_hash: &[u8]) -> String {
    let (pub_key, pri_key) = get_or_init_client_key_pair().unwrap();
    let expire_time = get_server_time() + Duration::seconds(10);
    let client_id = ClientId::new(pub_key.get_string().into(), expire_time);
    let client_id = client_id
        .encode(|client_id_data| {
            let sign_source = [api, body_hash, &client_id_data].concat();
            let signature = pri_key.sign(sign_source.as_slice()).unwrap();
            return Ok(signature);
        })
        .unwrap();
    return client_id;
}

async fn build_request(api: &str, body: &str) -> Request {
    let body_hash = sha512(body.as_bytes());
    let client_id = calc_client_id(api.as_bytes(), &body_hash);
    let mut opts = RequestInit::new();
    opts.method("POST");
    let mut headers: HashMap<&str, String> = HashMap::new();
    headers.insert(
        "Content-Type",
        "application/json; charset=utf-8".to_string(),
    );
    headers.insert("X-Client-Id", client_id);
    let body_hash: String = BASE64_STANDARD.encode(&body_hash);
    headers.insert("X-Hash", body_hash);
    opts.headers(&serde_wasm_bindgen::to_value(&headers).expect("Failed to serialize headers."));
    opts.body(Some(&JsValue::from_str(body)));
    let request = Request::new_with_str_and_init(api, &opts).expect("Failed to build request.");
    return request;
}

#[derive(Serialize, Deserialize)]
pub struct KeyPair {
    pub public: String,
    pub private: String,
}

fn get_or_init_client_key_pair() -> Result<(RsaPubKey2048, RsaPriKey2048), LightString> {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let client_key_pair = local_storage.get_item("clientKeyPair").unwrap();
    if let Some(client_key_pair) = client_key_pair {
        let key_pair =
            serde_json::from_str::<KeyPair>(&client_key_pair).map_err(|err| -> LightString {
                log::error!("密钥对格式不正确: {}", err);
                return "生成密钥对失败".into();
            })?;
        let pub_key = RsaPubKey2048::try_from_string(&key_pair.public);
        let pri_key = RsaPriKey2048::try_from_string(&key_pair.private);
        if is_valid_rsa_key_pair(&pub_key, &pri_key) {
            return Ok((pub_key, pri_key));
        } else {
            return Err(LightString::from("公私钥密钥对不匹配!"));
        }
    } else {
        let (pub_key, pri_key) = js::gen_rsa_key_pair();
        let key_pair = serde_json::to_string(&KeyPair {
            public: pub_key.get_string(),
            private: pri_key.get_string(),
        })
        .unwrap();
        local_storage.set_item("clientKeyPair", &key_pair).unwrap();
        return Ok((pub_key, pri_key));
    }
}

async fn ajax_inner(request: &Request) -> Result<LightString, LightString> {
    let window = web_sys::window().ok_or_else(|| -> LightString {
        log::error!("获取窗口对象失败");
        return LightString::Static("获取窗口对象失败");
    })?;
    let resp_value = JsFuture::from(window.fetch_with_request(request))
        .await
        .map_err(|err| -> LightString {
            log::error!("请求接口错误: {:?}", err);
            return LightString::Static("请求接口错误");
        })?;
    let response: Response = resp_value.dyn_into().unwrap();
    if !response.ok() {
        log::error!("响应状态码错误：{}", response.status());
        return Err(LightString::Static("响应状态码错误"));
    } else {
        let text = response.text().map_err(|err| -> LightString {
            log::error!("响应数据编码格式不正确: {:?}", err);
            return LightString::Static("响应数据编码格式不正确");
        })?;
        let body = JsFuture::from(text).await.map_err(|err| -> LightString {
            log::error!("响应数据编码格式不正确: {:?}", err);
            return LightString::Static("响应数据编码格式不正确");
        })?;
        return Ok(body.as_string().unwrap().into());
    }
}

pub async fn default_http_requestor(
    (url, req): (LightString, LightString),
) -> Result<LightString, LightString> {
    let request = build_request(&url, &req).await;
    return ajax_inner(&request).await;
}

impl request::Handler<bool, bool> for UseStateHandle<bool> {
    fn handle(&self, lock: bool) -> Pin<Box<dyn Future<Output = bool>>> {
        let result = if lock {
            if *self.deref() {
                false
            } else {
                self.set(true);
                true
            }
        } else {
            self.set(false);
            true
        };
        Box::pin(async move { result })
    }
}

pub async fn wait(millis: u32) {
    let mut timeout = None;
    let mut promise_fn = |resolve: Function, _reject: Function| {
        timeout.replace(Timeout::new(millis, move || {
            resolve.call0(&wasm_bindgen::JsValue::UNDEFINED).unwrap();
        }));
    };
    let promise = Promise::new(&mut promise_fn);
    JsFuture::from(promise).await.unwrap();
    timeout.take();
}

pub async fn alert(content: LightString, title: Option<LightString>) {
    let mut promise_fn = move |resolve: Function, _reject: Function| {
        components::alert::alert(
            content.clone(),
            title.clone(),
            Some(move || {
                resolve.call0(&wasm_bindgen::JsValue::UNDEFINED).unwrap();
            }),
        );
    };
    let promise = Promise::new(&mut promise_fn);
    JsFuture::from(promise).await.unwrap();
}

pub async fn confirm(content: LightString, title: Option<LightString>) -> bool {
    let mut promise_fn = move |resolve: Function, _reject: Function| {
        components::confirm::confirm(content.clone(), title.clone(), move |confirm| {
            resolve
                .call1(
                    &wasm_bindgen::JsValue::UNDEFINED,
                    &wasm_bindgen::JsValue::from_bool(confirm),
                )
                .unwrap();
        });
    };
    let promise = Promise::new(&mut promise_fn);
    let ret = JsFuture::from(promise).await.unwrap();
    return ret.as_bool().unwrap();
}

#[derive(PartialEq, Clone, Copy)]
pub enum LoadStatus {
    NotStarted,
    Loading,
    LoadFailed,
    LoadOk,
}

pub fn gen_id() -> String {
    unsafe {
        return super::ID_GEN.as_mut().unwrap().generate();
    }
}

pub fn empty_html() -> Html {
    html! {}
}

pub fn fill_random_bytes(data: &mut [u8]) {
    for ch in data.iter_mut() {
        *ch = (u8::MAX as f64 * random()).round() as u8;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadResp {
    pub key: String,
}

pub async fn upload_file(
    file: web_sys::File,
    sha512_promise: Promise,
    on_upload_progress: Option<Box<dyn FnMut(f64, f64)>>,
) -> Result<UploadResp, LightString> {
    let sha512 = JsFuture::from(sha512_promise)
        .await
        .map_err(|err| -> LightString {
            log::error!("计算sha512错误: {:?}", err);
            return "计算sha512错误".into();
        })?;
    let get_raw_method: js_sys::Function =
        js_sys::Reflect::get(&sha512, &JsValue::from_str("getRaw"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let sha512: Uint8Array = get_raw_method.call0(&sha512).unwrap().dyn_into().unwrap();
    let body_hash = sha512.to_vec();
    let api = UPLOAD_API;
    let client_id = calc_client_id(api.as_bytes(), &body_hash);
    let form = FormData::new().map_err(|_err| LightString::from("Failed to create FormData."))?;
    form.append_with_str("size", &file.size().to_string())
        .expect("Failed to add field \"size\".");
    form.append_with_blob("file", file.as_ref())
        .expect("Failed to add field \"file\".");
    let mut headers: HashMap<String, Vec<String>> = HashMap::new();
    headers.insert("X-Client-Id".to_string(), vec![client_id]);
    let body_hash: String = BASE64_STANDARD.encode(&body_hash);
    headers.insert("X-Hash".to_string(), vec![body_hash]);
    let resp = ajax::ajax::<(), ajax::Json<CommonResponse<UploadResp>>>(
        String::from(api),
        Some(ajax::Request::Form(form)),
        Some(ajax::RequestOption {
            headers: Some(headers),
            on_upload_progress: on_upload_progress,
            ..Default::default()
        }),
    )
    .await
    .map_err(|err| LightString::from(err.to_string()))?;
    if 0 == resp.code {
        let data = resp.data.ok_or_else(|| {
            return LightString::from("没有返回数据！");
        })?;
        return Ok(UploadResp {
            key: format!("blob/{}", data.key),
        });
    } else {
        return Err(resp.message.to_string().into());
    }
}

static mut LAST_DOM: Option<(web_sys::HtmlInputElement, js_sys::Function)> = None;

fn clean_last_dom() {
    if let Some((file_dom, on_change)) = unsafe { &LAST_DOM } {
        file_dom
            .remove_event_listener_with_callback("change", on_change)
            .unwrap();
        let document = web_sys::window().unwrap().document().unwrap();
        //如果上一次上传用户点击了取消，则没有执行change回调，把上一次生成的节点移除掉
        document.body().unwrap().remove_child(file_dom).unwrap();
        unsafe {
            LAST_DOM.take();
        }
    }
}

pub fn choose_file(cb: impl Fn(Option<web_sys::FileList>) + 'static, accept: Option<String>) {
    clean_last_dom();
    let document = web_sys::window().unwrap().document().unwrap();
    let file_dom: web_sys::HtmlInputElement = document
        .create_element("input")
        .unwrap()
        .dyn_into()
        .unwrap();
    file_dom.style().set_property("display", "none").unwrap();
    file_dom.set_attribute("type", "file").unwrap();
    if let Some(accept) = accept {
        file_dom.set_attribute("accept", &accept).unwrap();
    }
    let file_dom_clone = file_dom.clone();
    let on_change = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        cb(file_dom_clone.files());
        clean_last_dom();
    }) as Box<dyn FnMut(web_sys::Event)>);
    let on_change: js_sys::Function = on_change.into_js_value().dyn_into().unwrap();
    file_dom
        .add_event_listener_with_callback("change", &on_change)
        .unwrap();
    //如果不添加到文档里，IE下不能弹出选择文件的对话框
    document.body().unwrap().append_child(&file_dom).unwrap();
    unsafe {
        LAST_DOM.replace((file_dom.clone(), on_change));
    }
    file_dom.click();
}

//手动触发resize事件
pub fn trigger_resize() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let event = document.create_event("HTMLEvents").unwrap();
    event.init_event("resize");
    window.dispatch_event(&event).unwrap();
}

pub fn move_up<T: Clone>(list: &UseStateHandle<Vec<T>>, index: usize) -> bool {
    if 0 < index && index < list.len() {
        let mut new_list = list.deref().clone();
        new_list.swap(index - 1, index);
        list.set(new_list);
        return true;
    } else {
        return false;
    }
}

pub fn move_down<T: Clone>(list: &UseStateHandle<Vec<T>>, index: usize) -> bool {
    if index + 1 < list.len() {
        let mut new_list = list.deref().clone();
        new_list.swap(index, index + 1);
        list.set(new_list);
        return true;
    } else {
        return false;
    }
}

pub fn remove_item<T: Clone>(list: &UseStateHandle<Vec<T>>, index: usize) -> bool {
    if index < list.len() {
        let mut new_list = list.deref().clone();
        new_list.remove(index);
        list.set(new_list);
        return true;
    } else {
        return false;
    }
}
