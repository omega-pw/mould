use leptos::{prelude::*, tachys::view::any_view::AnyViewState};
use send_wrapper::SendWrapper;
use std::ops::DerefMut;
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::JsCast;
use web_sys::Element;

#[component]
pub fn Loading(center_middle: bool, #[prop(default = 999)] z_index: u64) -> impl IntoView {
    if center_middle {
        let style = format!("width: 3em;height: 3em;position: fixed;left: 50%;top: 50%;-ms-transform: translateX(-50%) translateY(-50%);-moz-transform: translateX(-50%) translateY(-50%);-webkit-transform: translateX(-50%) translateY(-50%);-o-transform: translateX(-50%) translateY(-50%);transform: translateX(-50%) translateY(-50%);z-index: {};", z_index);
        view! {
            <div style={style}>
                <i class="loading" style="width: 100%;height: 100%;"></i>
            </div>
        }
        .into_any()
    } else {
        view! {
            <i class="loading" style="width: 3em;height: 3em;"></i>
        }
        .into_any()
    }
}

static INSTANCE: LazyLock<Mutex<Option<SendWrapper<(UnmountHandle<AnyViewState>, Element, u32)>>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn show() {
    let inst = LazyLock::force(&INSTANCE);
    if let Some(inst) = inst.lock().unwrap().as_mut() {
        inst.2 += 1;
        return;
    }
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let loading_root = document.create_element("div").unwrap();
    body.append_child(&loading_root).unwrap();
    let renderer = || {
        view! {
            <Loading center_middle=true z_index=999 />
        }
        .into_any()
    };
    let loading_handle = leptos::mount::mount_to(loading_root.clone().unchecked_into(), renderer);
    inst.lock()
        .unwrap()
        .replace(SendWrapper::new((loading_handle, loading_root, 1)));
}

fn destroy(inst: &mut Option<SendWrapper<(UnmountHandle<AnyViewState>, Element, u32)>>) {
    let inst = inst.take();
    if let Some(wrapper) = inst {
        let (loading_handle, loading_root, _) = wrapper.take();
        // loading_handle.destroy();
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        body.remove_child(&loading_root).unwrap();
    }
}

pub fn hide() {
    let inst = LazyLock::force(&INSTANCE);
    let mut inst = inst.lock().unwrap();
    if let Some(wrapper) = inst.as_mut() {
        if 1 >= wrapper.2 {
            destroy(inst.deref_mut());
        } else {
            wrapper.2 -= 1;
        }
    }
}
