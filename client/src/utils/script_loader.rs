use js_sys::Promise;
use send_wrapper::SendWrapper;
use std::ops::Deref;
use std::sync::RwLock;
use tihu::SharedString;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, window, HtmlScriptElement};

pub struct ScriptLoader {
    url: SharedString,
    global_var: SharedString,
    promise: RwLock<Option<SendWrapper<Promise>>>,
}

impl ScriptLoader {
    pub fn new(url: impl Into<SharedString>, global_var: impl Into<SharedString>) -> Self {
        Self {
            url: url.into(),
            global_var: global_var.into(),
            promise: RwLock::new(None),
        }
    }

    pub async fn get_or_load(&self) -> Result<JsValue, JsValue> {
        let win = window().ok_or("No window found")?;
        // 检查变量是否已存在
        let mut value = js_sys::Reflect::get(&win, &JsValue::from_str(&self.global_var))?;
        if value.is_undefined() || value.is_null() {
            let promise = self.promise.read().unwrap();
            let task = if let Some(promise) = promise.as_ref() {
                promise.deref().clone()
            } else {
                let new_promise = add_script(&self.url)?;
                drop(promise);
                self.promise
                    .write()
                    .unwrap()
                    .replace(SendWrapper::new(new_promise.clone()));
                new_promise
            };
            JsFuture::from(task).await?;
        }
        value = js_sys::Reflect::get(&win, &JsValue::from_str(&self.global_var))?;
        return Ok(value);
    }
}

fn add_script(url: &str) -> Result<Promise, JsValue> {
    let win = window().ok_or("No window found")?;
    // 动态创建脚本标签
    let document = win.document().ok_or("No document found")?;
    let script = document
        .create_element("script")?
        .dyn_into::<HtmlScriptElement>()?;
    script.set_type("text/javascript");
    script.set_src(url);
    // 封装 Promise 监听加载状态
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        script.set_onload(Some(&resolve));
        script.set_onerror(Some(&reject));
    });
    let head = document.head().ok_or("No head found")?;
    head.append_child(&script)?;
    return Ok(promise);
}
