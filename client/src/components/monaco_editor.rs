use super::on_cleanup_unsync;
use super::LatestDestroy;
use crate::SharedString;
use leptos::html;
use leptos::prelude::*;
use std::ops::Deref;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlElement};

#[component]
pub fn MonacoEditor(
    value: RwSignal<SharedString>,
    #[prop(into, default = None)] language: Option<SharedString>,
    #[prop(optional)] readonly: bool,
    #[prop(into, optional)] width: MaybeProp<SharedString>,
    #[prop(into, optional)] height: MaybeProp<SharedString>,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<SharedString>>,
) -> impl IntoView {
    let div_ref: NodeRef<html::Div> = NodeRef::new();
    let editor = RwSignal::new_local(None);
    let inner_value: RwSignal<SharedString> = RwSignal::new(Default::default());
    let on_change = {
        let value = value.clone();
        let inner_value = inner_value.clone();
        UnsyncCallback::new(move |new_value: SharedString| {
            inner_value.set(new_value.clone());
            if !readonly {
                value.set(new_value.clone());
            }
            if let Some(onchange) = onchange {
                onchange.run(new_value);
            }
        })
    };
    let latest_destroy = LatestDestroy::new();
    on_cleanup_unsync({
        let latest_destroy = latest_destroy.clone();
        move || {
            latest_destroy.clear();
        }
    });
    {
        let div_ref = div_ref.clone();
        let value = value.clone();
        let editor = editor.clone();
        Effect::watch(
            move || div_ref.get(),
            move |div_ref, _, _| {
                if let Some(div) = div_ref {
                    let (editor_inst, subscription) =
                        init_monaco_editor(&div, &value.read(), &language, readonly, on_change);
                    editor.set(Some(editor_inst.clone()));
                    let editor_inst_clone = editor_inst.clone();
                    let on_resize = Closure::wrap(Box::new(move |_event: Event| {
                        update_editor(&editor_inst_clone);
                    }) as Box<dyn Fn(Event)>)
                    .into_js_value();
                    let window = web_sys::window().unwrap();
                    window
                        .add_event_listener_with_callback("resize", on_resize.unchecked_ref())
                        .unwrap();
                    latest_destroy.replace(move || {
                        window
                            .remove_event_listener_with_callback(
                                "resize",
                                on_resize.unchecked_ref(),
                            )
                            .unwrap();
                        destroy_monaco(&editor_inst, &subscription);
                    });
                }
            },
            false,
        );
    }
    {
        let value = value.clone();
        let editor = editor.clone();
        Effect::watch(
            move || value.get(),
            move |value, _, _| {
                let inner_value = inner_value.read();
                if inner_value.deref() != value {
                    if let Some(editor) = editor.read().as_ref() {
                        update_value(editor, value).unwrap();
                    }
                }
            },
            false,
        );
    }
    {
        let width = width.clone();
        let height = height.clone();
        let editor = editor.clone();
        Effect::watch(
            move || (width.get(), height.get()),
            move |_, _, _| {
                if let Some(editor) = editor.read().as_ref() {
                    update_editor(editor);
                }
            },
            false,
        );
    }
    let style = move || {
        let mut style = String::new();
        if let Some(width) = width.read().as_ref() {
            style.push_str(&format!("width: {};", width));
        }
        if let Some(height) = height.read().as_ref() {
            style.push_str(&format!("height: {};", height));
        }
        if style.is_empty() {
            None
        } else {
            Some(style)
        }
    };
    view! {
        <div style={style} node_ref={div_ref}/>
    }
}

pub fn init_monaco_editor(
    root: &HtmlElement,
    value: &SharedString,
    language: &Option<SharedString>,
    readonly: bool,
    onchange: UnsyncCallback<SharedString>,
) -> (JsValue, JsValue) {
    let monaco =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("monaco")).unwrap();
    let editor = js_sys::Reflect::get(&monaco, &JsValue::from_str("editor")).unwrap();
    let create_method: js_sys::Function =
        js_sys::Reflect::get(&editor, &JsValue::from_str("create"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let init_config = js_sys::Object::new();
    js_sys::Reflect::set(
        &init_config,
        &JsValue::from_str("value"),
        &JsValue::from_str(value.as_ref()),
    )
    .unwrap();
    if let Some(language) = language.as_ref() {
        js_sys::Reflect::set(
            &init_config,
            &JsValue::from_str("language"),
            &JsValue::from_str(language.as_ref()),
        )
        .unwrap();
    }
    js_sys::Reflect::set(
        &init_config,
        &JsValue::from_str("selectOnLineNumbers"),
        &JsValue::from_bool(true),
    )
    .unwrap();
    js_sys::Reflect::set(
        &init_config,
        &JsValue::from_str("readOnly"),
        &JsValue::from_bool(readonly),
    )
    .unwrap();
    js_sys::Reflect::set(
        &init_config,
        &JsValue::from_str("theme"),
        &JsValue::from_str("vs-dark"),
    )
    .unwrap();
    let editor_inst = create_method.call2(&editor, root, &init_config).unwrap();
    let on_did_change_model_content_method: js_sys::Function =
        js_sys::Reflect::get(&editor_inst, &JsValue::from_str("onDidChangeModelContent"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let editor_inst_clone = editor_inst.clone();
    let on_change = Closure::wrap(Box::new(move |_event: Event| {
        let get_value_method: js_sys::Function =
            js_sys::Reflect::get(&editor_inst_clone, &JsValue::from_str("getValue"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let value = get_value_method.call0(&editor_inst_clone).unwrap();
        if let Some(value) = value.as_string() {
            onchange.run(value.into());
        }
    }) as Box<dyn Fn(Event)>)
    .into_js_value();
    let subscription = on_did_change_model_content_method
        .call1(&editor_inst, &on_change)
        .unwrap();
    return (editor_inst, subscription);
}

pub fn update_value(editor_inst: &JsValue, value: &str) -> Result<(), JsValue> {
    let set_value_method: js_sys::Function =
        js_sys::Reflect::get(editor_inst, &JsValue::from_str("setValue"))
            .unwrap()
            .dyn_into()
            .unwrap();
    set_value_method
        .call1(editor_inst, &JsValue::from_str(value))
        .unwrap();
    return Ok(());
}

pub fn destroy_monaco(editor_inst: &JsValue, subscription: &JsValue) {
    let dispose_method: js_sys::Function =
        js_sys::Reflect::get(editor_inst, &JsValue::from_str("dispose"))
            .unwrap()
            .dyn_into()
            .unwrap();
    dispose_method.call0(editor_inst).unwrap();
    let get_model_method: js_sys::Function =
        js_sys::Reflect::get(editor_inst, &JsValue::from_str("getModel"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let model = get_model_method.call0(editor_inst).unwrap();
    if !model.is_null() {
        let dispose_method: js_sys::Function =
            js_sys::Reflect::get(&model, &JsValue::from_str("dispose"))
                .unwrap()
                .dyn_into()
                .unwrap();
        dispose_method.call0(&model).unwrap();
    }
    let dispose_method: js_sys::Function =
        js_sys::Reflect::get(&subscription, &JsValue::from_str("dispose"))
            .unwrap()
            .dyn_into()
            .unwrap();
    dispose_method.call0(&subscription).unwrap();
}

pub fn update_editor(editor: &JsValue) {
    let layout_method: js_sys::Function =
        js_sys::Reflect::get(editor, &JsValue::from_str("layout"))
            .unwrap()
            .dyn_into()
            .unwrap();
    layout_method.call0(editor).unwrap();
}
