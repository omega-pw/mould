use super::button::Button;
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
pub fn Alert(
    #[prop(into, default = Signal::stored(SharedString::from("提示")))] title: Signal<SharedString>,
    content: SharedString,
    #[prop(default = 999)] z_index: u64,
    #[prop(into, default = SharedString::from("确定"))] ok_text: SharedString,
    #[prop(into, default = None)] onok: Option<UnsyncCallback<()>>,
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
    let on_click = UnsyncCallback::new(move |_| {
        if let Some(onok) = onok.as_ref() {
            onok.run(());
        }
    });
    view! {
        <ModalDialog title={title.clone()} closable=false z_index={z_index} center_style={center_style}>
            <div style="min-height: 2em;padding:0.5em;">
                {content}
            </div>
            <div style={footer_style}>
                <Button onclick={on_click} style={SharedString::from(btn_style)}>{ok_text.clone()}</Button>
            </div>
        </ModalDialog>
    }
}

pub fn alert(content: SharedString, title: Option<SharedString>, cb: Option<impl Fn() + 'static>) {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let alert_root = document.create_element("div").unwrap();
    let on_root_click: Function = Closure::wrap(Box::new(|event: Event| {
        //阻止mousedown事件冒泡，防止点击事件被document补货到，导致FocusArea组件触发离开事件
        event.stop_propagation();
    }) as Box<dyn FnMut(Event)>)
    .into_js_value()
    .dyn_into()
    .unwrap();
    alert_root
        .add_event_listener_with_callback("mousedown", &on_root_click)
        .unwrap();
    body.append_child(&alert_root).unwrap();
    let alert_handle: Rc<Cell<Option<UnmountHandle<AnyViewState>>>> = Rc::new(Cell::new(None));
    let alert_handle_clone = alert_handle.clone();
    let alert_root_clone = alert_root.clone();
    let renderer = || {
        view! {
            <Alert
                title={title.unwrap_or_else(|| SharedString::from("提示"))}
                content=content
                z_index=999
                ok_text=SharedString::from("确定")
                onok=UnsyncCallback::new(move |_: ()| {
                    if let Some(cb) = cb.as_ref() {
                        cb();
                    }
                    if let Some(alert_handle) = alert_handle_clone.take() {
                        // alert_handle.destroy();
                        let document = web_sys::window().unwrap().document().unwrap();
                        let body = document.body().unwrap();
                        body.remove_child(&alert_root_clone).unwrap();
                    }
                    alert_root_clone
                        .remove_event_listener_with_callback("mousedown", &on_root_click)
                        .unwrap();
                })
            />
        }
        .into_any()
    };
    alert_handle.set(Some(leptos::mount::mount_to(
        alert_root.unchecked_into(),
        renderer,
    )));
}
