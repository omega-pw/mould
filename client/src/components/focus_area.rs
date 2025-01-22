use gloo::timers::callback::Timeout;
use js_sys::Reflect;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::MouseEvent;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub onclickother: Option<Callback<()>>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    pub children: Children,
}

fn get_or_init_tags(object: &JsValue, tag_key: &JsValue) -> JsValue {
    let tags = Reflect::get(object, tag_key).unwrap();
    if tags.is_object() {
        return tags;
    } else {
        let tags = js_sys::Object::new();
        Reflect::set(object, tag_key, &tags).unwrap();
        return tags.into();
    }
}

fn has_tag(object: &JsValue, tag_key: &JsValue, tag_id: &JsValue) -> bool {
    let tags = Reflect::get(object, tag_key).unwrap();
    if tags.is_object() {
        let value = Reflect::get(&tags, tag_id).unwrap();
        if let Some(value) = value.as_bool() {
            return value;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

#[function_component]
pub fn FocusArea(props: &Props) -> Html {
    let tag_key = js_sys::Symbol::for_("EventTag");
    let tag_id: UseStateHandle<JsValue> = use_state(|| JsValue::symbol(Some("FocusArea")));
    let onclickother = props.onclickother.clone();
    let tag_key_clone = tag_key.clone();
    let tag_id_clone = tag_id.clone();
    use_effect_with((), move |_| {
        let document = web_sys::window().unwrap().document().unwrap();
        let onclick = Closure::wrap(Box::new(move |evt: MouseEvent| {
            if let Some(onclickother) = onclickother.as_ref() {
                if !has_tag(&evt, &tag_key_clone, &tag_id_clone) {
                    onclickother.emit(());
                }
            }
        }) as Box<dyn Fn(MouseEvent)>)
        .into_js_value();
        let document_clone = document.clone();
        let onclick_clone = onclick.clone();
        let mut timeout = Some(Timeout::new(0, move || {
            //延迟一点儿时间再绑定，不然如果过早绑定，点击打开按钮的事件也被处理导致无法打开
            document_clone
                .add_event_listener_with_callback("mousedown", onclick_clone.unchecked_ref())
                .unwrap();
        }));
        move || {
            timeout.take();
            document
                .remove_event_listener_with_callback("mousedown", onclick.unchecked_ref())
                .unwrap();
        }
    });
    let on_area_click = Callback::from(move |evt: MouseEvent| {
        let tags: JsValue = get_or_init_tags(&evt, &tag_key);
        Reflect::set(&tags, &tag_id, &JsValue::from_bool(true)).unwrap();
    });
    html! {
        <div style={props.style.clone()} onmousedown={on_area_click}>
            {props.children.clone()}
        </div>
    }
}
