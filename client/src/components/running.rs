use gloo::timers::callback::Timeout;
use js_sys::Function;
use js_sys::Promise;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or(AttrValue::from("..."))]
    pub text: AttrValue,
    #[prop_or(500)]
    pub step: u32,
}

#[function_component]
pub fn Running(props: &Props) -> Html {
    let curr_text: UseStateHandle<String> = use_state(|| String::from(""));
    let curr_text_clone = curr_text.clone();
    let text = props.text.clone();
    let step = props.step;
    use_effect_with((text.clone(), step), move |_| {
        let destroyed = Arc::new(AtomicBool::new(false));
        let destroyed_clone = destroyed.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_loop(&curr_text_clone, &text, step, &destroyed_clone).await;
        });
        move || {
            destroyed.store(true, Ordering::Relaxed);
        }
    });
    html! { curr_text.deref().clone() }
}

async fn start_loop(
    curr_text: &UseStateHandle<String>,
    text: &str,
    step: u32,
    destroyed: &AtomicBool,
) {
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
