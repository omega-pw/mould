use crate::SharedString;
use gloo::timers::callback::Timeout;
use js_sys::Function;
use js_sys::Promise;
use leptos::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn Running(
    #[prop(into, default = SharedString::from("..."))] text: SharedString,
    #[prop(default = 500)] step: u32,
) -> impl IntoView {
    let curr_text: RwSignal<String> = RwSignal::new(String::from(""));
    let curr_text_clone = curr_text.clone();
    let destroyed = Arc::new(AtomicBool::new(false));
    let destroyed_clone = destroyed.clone();
    wasm_bindgen_futures::spawn_local(async move {
        start_loop(&curr_text_clone, &text, step, &destroyed_clone).await;
    });
    on_cleanup(move || {
        destroyed.store(true, Ordering::Relaxed);
    });
    curr_text.into_any()
}

async fn start_loop(curr_text: &RwSignal<String>, text: &str, step: u32, destroyed: &AtomicBool) {
    let mut max_char_count: usize = 0;
    let char_count = text.chars().count();
    loop {
        if destroyed.load(Ordering::Relaxed) {
            break;
        } else {
            curr_text.set(text.chars().take(max_char_count + 1).collect());
            max_char_count = (max_char_count + 1) % char_count;
            wait(step).await;
        }
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
