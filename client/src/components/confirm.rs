use super::button::Button;
use super::button_group::ButtonGroup;
use super::modal_dialog::ModalDialog;
use crate::SharedString;
use js_sys::Function;
use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyViewState;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Event;

#[component]
pub fn Confirm(
    #[prop(into, default = Signal::stored(SharedString::from("确认")))] title: Signal<SharedString>,
    content: SharedString,
    #[prop(default = 999)] z_index: u64,
    #[prop(into, default = SharedString::from("确定"))] ok_text: SharedString,
    #[prop(into, default = SharedString::from("取消"))] cancel_text: SharedString,
    ondone: UnsyncCallback<bool>,
) -> impl IntoView {
    let is_mobile = false;
    let center_style = if is_mobile {
        "min-width:40%;max-width:60%;"
    } else {
        "min-width:24em;max-width:48em;"
    };
    let mut footer_style = String::from("text-align:center;");
    if !is_mobile {
        footer_style.push_str("padding-bottom:0.5em;");
    }
    let mut btn_style = String::from("font-size: inherit;");
    if is_mobile {
        btn_style.push_str("display:block;width:100%;background-color:transparent;border:none;border-top-width:1px;border-top-style:solid;border-top-color:#EEEEEE");
    }
    let btn_style = SharedString::from(btn_style);
    let on_ok = UnsyncCallback::new({
        let ondone = ondone.clone();
        move |_| {
            ondone.run(true);
        }
    });
    let on_cancel = UnsyncCallback::new(move |_| {
        ondone.run(false);
    });
    view! {
        <ModalDialog title={title} closable=false z_index={z_index} center_style={center_style}>
            <div style="min-height: 2em;padding:0.5em;">{content}</div>
            <div style={footer_style}>
                <ButtonGroup>
                    <Button onclick={on_ok} style={btn_style.clone()}>{ok_text}</Button>
                    <Button onclick={on_cancel} style={btn_style}>{cancel_text}</Button>
                </ButtonGroup>
            </div>
        </ModalDialog>
    }
}

pub fn confirm(content: SharedString, title: Option<SharedString>, cb: impl Fn(bool) + 'static) {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let confirm_root = document.create_element("div").unwrap();
    let on_root_click: Function = Closure::wrap(Box::new(|event: Event| {
        //阻止mousedown事件冒泡，防止点击事件被document补货到，导致FocusArea组件触发离开事件
        event.stop_propagation();
    }) as Box<dyn FnMut(Event)>)
    .into_js_value()
    .dyn_into()
    .unwrap();
    confirm_root
        .add_event_listener_with_callback("mousedown", &on_root_click)
        .unwrap();
    body.append_child(&confirm_root).unwrap();
    let confirm_handle: Rc<Cell<Option<UnmountHandle<AnyViewState>>>> = Rc::new(Cell::new(None));
    let confirm_handle_clone = confirm_handle.clone();
    let confirm_root_clone = confirm_root.clone();
    let renderer = || {
        view! {
            <Confirm
                title=title.unwrap_or_else(|| SharedString::from("确认"))
                content=content
                z_index=999
                ok_text=SharedString::from("确定")
                cancel_text=SharedString::from("取消")
                ondone=UnsyncCallback::new(move |ret: bool| {
                    cb(ret);
                    if let Some(confirm_handle) = confirm_handle_clone.take() {
                        // confirm_handle.destroy();
                        let document = web_sys::window().unwrap().document().unwrap();
                        let body = document.body().unwrap();
                        body.remove_child(&confirm_root_clone).unwrap();
                    }
                    confirm_root_clone
                        .remove_event_listener_with_callback("mousedown", &on_root_click)
                        .unwrap();
                })
            />
        }
        .into_any()
    };
    confirm_handle.set(Some(leptos::mount::mount_to(
        confirm_root.unchecked_into(),
        renderer,
    )));
}
