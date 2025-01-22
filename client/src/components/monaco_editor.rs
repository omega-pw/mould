use std::ops::Deref;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlElement};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: AttrValue,
    #[prop_or_default]
    pub language: Option<AttrValue>,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub width: Option<AttrValue>,
    #[prop_or_default]
    pub height: Option<AttrValue>,
    pub onchange: Callback<AttrValue>,
}

#[function_component]
pub fn MonacoEditor(props: &Props) -> Html {
    let div_ref = use_node_ref();
    let editor = use_state(|| None);
    let inner_value: UseStateHandle<AttrValue> = use_state(Default::default);
    let onchange = props.onchange.clone();
    let on_change = {
        let inner_value = inner_value.clone();
        Callback::from(move |value: AttrValue| {
            inner_value.set(value.clone());
            onchange.emit(value);
        })
    };
    {
        let div_ref = div_ref.clone();
        let value = props.value.clone();
        let language = props.language.clone();
        let readonly = props.readonly;
        let editor = editor.clone();
        use_effect_with(div_ref, move |div_ref| {
            let div = div_ref
                .cast::<HtmlElement>()
                .expect("div_ref not attached to div element");
            let (editor_inst, subscription) =
                init_monaco_editor(&div, &value, &language, readonly, on_change);
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
            move || {
                window
                    .remove_event_listener_with_callback("resize", on_resize.unchecked_ref())
                    .unwrap();
                destroy_monaco(&editor_inst, &subscription);
            }
        });
    }
    {
        let value = props.value.clone();
        let editor = editor.clone();
        use_effect_with(value, move |value| {
            if inner_value.deref() != value {
                if let Some(editor) = editor.as_ref() {
                    update_value(editor, value).unwrap();
                }
            }
        });
    }
    {
        let width = props.width.clone();
        let height = props.height.clone();
        let editor = editor.clone();
        use_effect_with((width, height), move |_| {
            if let Some(editor) = editor.as_ref() {
                update_editor(editor);
            }
        });
    }
    let mut style = String::new();
    if let Some(width) = props.width.as_ref() {
        style.push_str(&format!("width: {};", width));
    }
    if let Some(height) = props.height.as_ref() {
        style.push_str(&format!("height: {};", height));
    }
    let style = if style.is_empty() { None } else { Some(style) };
    html! {
        <div style={style} ref={div_ref}/>
    }
}

pub fn init_monaco_editor(
    root: &HtmlElement,
    value: &AttrValue,
    language: &Option<AttrValue>,
    readonly: bool,
    onchange: Callback<AttrValue>,
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
            onchange.emit(value.into());
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

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps {
    pub value: UseStateHandle<AttrValue>,
    #[prop_or_default]
    pub language: Option<AttrValue>,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub width: Option<AttrValue>,
    #[prop_or_default]
    pub height: Option<AttrValue>,
    #[prop_or_default]
    pub onchange: Option<Callback<AttrValue>>,
}

#[function_component]
pub fn BindingMonacoEditor(props: &BindingProps) -> Html {
    let value_clone = props.value.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |value: AttrValue| {
        value_clone.set(value.clone().into());
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(value);
        }
    });
    return html! {
        <MonacoEditor
            value={props.value.deref().clone()}
            language={props.language.clone()}
            readonly={props.readonly.clone()}
            width={props.width.clone()}
            height={props.height.clone()}
            onchange={on_change}
        />
    };
}
