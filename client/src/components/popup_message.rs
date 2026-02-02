use super::common_popup::CommonPopup;
use crate::SharedString;
use gloo::timers::callback::Timeout;
use js_sys::Function;
use js_sys::Promise;
use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyViewState;
use send_wrapper::SendWrapper;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Element;

#[derive(Clone, PartialEq)]
pub enum MsgType {
    Success,
    Warning,
    Error,
}

#[component]
pub fn PopupMessage(
    msg_type: MsgType,
    message: SharedString,
    #[prop(default = None)] z_index: Option<i32>,
    #[prop(default = None)] duration: Option<u32>,
    #[prop(into, default = None)] ondone: Option<UnsyncCallback<()>>,
) -> impl IntoView {
    let active = RwSignal::new(false);
    let destroyed = Arc::new(AtomicBool::new(false));
    wasm_bindgen_futures::spawn_local({
        let active = active.clone();
        let destroyed: Arc<AtomicBool> = destroyed.clone();
        async move {
            start_animation(&active, duration, &destroyed, &ondone).await;
        }
    });
    on_cleanup(move || {
        destroyed.store(true, Ordering::Relaxed);
    });
    let mut style = None;
    if let Some(z_index) = z_index {
        style.replace(format!("z-index: {}", z_index));
    }
    let class_suffix = match msg_type {
        MsgType::Success => "success",
        MsgType::Warning => "warning",
        MsgType::Error => "error",
    };
    view! {
        <CommonPopup active={Signal::from(active)} z_index={z_index}>
            <div class={format!("msg-{}", class_suffix)}>{ message }</div>
        </CommonPopup>
    }
}

async fn start_animation(
    active: &RwSignal<bool>,
    duration: Option<u32>,
    destroyed: &AtomicBool,
    ondone: &Option<UnsyncCallback<()>>,
) {
    active.set(true);
    wait(duration.unwrap_or(3000)).await;
    active.set(false);
    if destroyed.load(Ordering::Relaxed) {
        return;
    }
    wait(duration.unwrap_or(500)).await;
    if destroyed.load(Ordering::Relaxed) {
        return;
    }
    if let Some(ondone) = ondone.as_ref() {
        ondone.run(());
    }
}

pub async fn wait(millis: u32) {
    let mut timeout = None;
    let mut promise_fn = |resolve: Function, _reject: Function| {
        timeout.replace(Timeout::new(millis, move || {
            resolve.call0(&JsValue::UNDEFINED).unwrap();
        }));
    };
    let promise = Promise::new(&mut promise_fn);
    JsFuture::from(promise).await.unwrap();
    timeout.take();
}

static INSTANCE: LazyLock<Mutex<Option<SendWrapper<(UnmountHandle<AnyViewState>, Element)>>>> =
    LazyLock::new(|| Mutex::new(None));

fn destroy() {
    let inst = LazyLock::force(&INSTANCE);
    let inst = inst.lock().unwrap().take();
    if let Some(wrapper) = inst {
        let (popup_message_handle, popup_message_root) = wrapper.take();
        // popup_message_handle.destroy();
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        body.remove_child(&popup_message_root).unwrap();
    }
}

pub fn show(msg_type: MsgType, message: SharedString, duration: Option<u32>, z_index: Option<i32>) {
    destroy();
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let popup_message_root = document.create_element("div").unwrap();
    body.append_child(&popup_message_root).unwrap();
    let ondone = UnsyncCallback::new(move |_: ()| {
        destroy();
    });
    let renderer = move || {
        view! {
            <PopupMessage
                msg_type=msg_type
                message=message
                z_index=Some(z_index.unwrap_or(999))
                duration=duration
                ondone=Some(ondone)
            />
        }
        .into_any()
    };
    let popup_message_handle =
        leptos::mount::mount_to(popup_message_root.clone().unchecked_into(), renderer);
    let inst = LazyLock::force(&INSTANCE);
    inst.lock()
        .unwrap()
        .replace(SendWrapper::new((popup_message_handle, popup_message_root)));
}

pub fn success(message: SharedString) {
    show(MsgType::Success, message, None, None);
}

pub fn warning(message: SharedString) {
    show(MsgType::Warning, message, None, None);
}

pub fn error(message: SharedString) {
    show(MsgType::Error, message, None, None);
}
