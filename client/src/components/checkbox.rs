use std::ops::Deref;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: bool,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub onchange: Option<Callback<bool>>,
}

#[function_component]
pub fn Checkbox(props: &Props) -> Html {
    let value = props.value;
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |evt: Event| {
        let input: HtmlInputElement = evt.target_unchecked_into();
        input.set_checked(value);
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(!value);
        }
    });
    if let Some(label) = props.label.as_ref() {
        html! {
            <label>
                <input type="checkbox" checked={value} onchange={on_change} />
                {label}
            </label>
        }
    } else {
        html! {
            <input type="checkbox" checked={value} onchange={on_change} />
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps {
    pub value: UseStateHandle<bool>,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub onchange: Option<Callback<bool>>,
}

#[function_component]
pub fn BindingCheckbox(props: &BindingProps) -> Html {
    let value_clone = props.value.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |new_value: bool| {
        value_clone.set(new_value);
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(new_value);
        }
    });
    return html! {
        <Checkbox
            value={props.value.deref().clone()}
            label={props.label.clone()}
            onchange={on_change}
        />
    };
}
