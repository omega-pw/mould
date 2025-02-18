use std::ops::Deref;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or(AttrValue::from("text"))]
    pub r#type: AttrValue,
    pub value: AttrValue,
    #[prop_or_default]
    pub disable_trim: bool,
    #[prop_or_default]
    pub tabindex: Option<i32>,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    #[prop_or_default]
    pub oninput: Option<Callback<InputEvent>>,
    #[prop_or_default]
    pub onchange: Option<Callback<Event>>,
    #[prop_or_default]
    pub onupdate: Option<Callback<AttrValue>>,
    #[prop_or_default]
    pub onfocus: Option<Callback<()>>,
    #[prop_or_default]
    pub onblur: Option<Callback<()>>,
    #[prop_or_default]
    pub onkeydown: Option<Callback<KeyboardEvent>>,
    #[prop_or_default]
    pub onkeyup: Option<Callback<KeyboardEvent>>,
    #[prop_or_default]
    pub onenter: Option<Callback<()>>,
}

#[function_component]
pub fn Input(props: &Props) -> Html {
    let disable_trim = props.disable_trim;
    let oninput = props.oninput.clone();
    let onupdate = props.onupdate.clone();
    let on_input = Callback::from(move |evt: InputEvent| {
        let input: HtmlInputElement = evt.target_unchecked_into();
        let value = input.value();
        if let Some(onupdate) = onupdate.as_ref() {
            onupdate.emit(value.into());
        }
        if let Some(oninput) = oninput.as_ref() {
            oninput.emit(evt);
        }
    });
    let onchange = props.onchange.clone();
    let onupdate = props.onupdate.clone();
    let on_change = Callback::from(move |evt: Event| {
        if let Some(onupdate) = onupdate.as_ref() {
            let input: HtmlInputElement = evt.target_unchecked_into();
            let value = input.value();
            let value = if disable_trim {
                value
            } else {
                value.trim().to_string()
            };
            onupdate.emit(value.into());
        }
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(evt);
        }
    });
    let onfocus = props.onfocus.clone();
    let on_focus = Callback::from(move |_| {
        if let Some(onfocus) = onfocus.as_ref() {
            onfocus.emit(());
        }
    });
    let onblur = props.onblur.clone();
    let on_blur = Callback::from(move |_| {
        if let Some(onblur) = onblur.as_ref() {
            onblur.emit(());
        }
    });
    let onkeydown = props.onkeydown.clone();
    let on_keydown = Callback::from(move |evt: KeyboardEvent| {
        if let Some(onkeydown) = onkeydown.as_ref() {
            onkeydown.emit(evt);
        }
    });
    let onkeyup = props.onkeyup.clone();
    let onenter = props.onenter.clone();
    let on_keyup = Callback::from(move |evt: KeyboardEvent| {
        let key_code = evt.key_code();
        if let Some(onkeyup) = onkeyup.as_ref() {
            onkeyup.emit(evt);
        }
        if 13 == key_code {
            if let Some(onenter) = onenter.as_ref() {
                onenter.emit(());
            }
        }
    });
    html! {
        <input
            type={props.r#type.clone()}
            class="e-input"
            value={props.value.clone()}
            tabindex={props.tabindex.map(|tabindex|tabindex.to_string())}
            placeholder={props.placeholder.clone()}
            style={props.style.clone()}
            oninput={on_input}
            onchange={on_change}
            onfocus={on_focus}
            onblur={on_blur}
            onkeydown={on_keydown}
            onkeyup={on_keyup}
        />
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps {
    #[prop_or(AttrValue::from("text"))]
    pub r#type: AttrValue,
    pub value: UseStateHandle<AttrValue>,
    #[prop_or_default]
    pub disable_trim: bool,
    #[prop_or_default]
    pub tabindex: Option<i32>,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    #[prop_or_default]
    pub oninput: Option<Callback<InputEvent>>,
    #[prop_or_default]
    pub onchange: Option<Callback<Event>>,
    #[prop_or_default]
    pub onupdate: Option<Callback<AttrValue>>,
    #[prop_or_default]
    pub onfocus: Option<Callback<()>>,
    #[prop_or_default]
    pub onblur: Option<Callback<()>>,
    #[prop_or_default]
    pub onkeydown: Option<Callback<KeyboardEvent>>,
    #[prop_or_default]
    pub onkeyup: Option<Callback<KeyboardEvent>>,
    #[prop_or_default]
    pub onenter: Option<Callback<()>>,
}

#[function_component]
pub fn BindingInput(props: &BindingProps) -> Html {
    let value_clone = props.value.clone();
    let onupdate = props.onupdate.clone();
    let on_update = Callback::from(move |value: AttrValue| {
        value_clone.set(value.clone());
        if let Some(onupdate) = onupdate.as_ref() {
            onupdate.emit(value);
        }
    });
    return html! {
        <Input
            r#type={props.r#type.clone()}
            value={props.value.deref().clone()}
            tabindex={props.tabindex.clone()}
            placeholder={props.placeholder.clone()}
            style={props.style.clone()}
            oninput={props.oninput.clone()}
            onchange={props.onchange.clone()}
            onupdate={on_update}
            onfocus={props.onfocus.clone()}
            onblur={props.onblur.clone()}
            onkeydown={props.onkeydown.clone()}
            onkeyup={props.onkeyup.clone()}
            onenter={props.onenter.clone()}
        />
    };
}
