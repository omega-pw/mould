use crate::SharedString;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::HtmlInputElement;
use web_sys::KeyboardEvent;

#[component]
pub fn Input(
    #[prop(into, default = SharedString::from("text"))] r#type: SharedString,
    value: RwSignal<SharedString>,
    #[prop(optional)] disable_trim: bool,
    #[prop(optional)] readonly: bool,
    #[prop(into, default = None)] tabindex: Option<i32>,
    #[prop(into, optional)] placeholder: SharedString,
    #[prop(into, optional)] style: SharedString,
    #[prop(into, default = None)] oninput: Option<UnsyncCallback<Event>>,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<Event>>,
    #[prop(into, default = None)] onupdate: Option<UnsyncCallback<SharedString>>,
    #[prop(into, default = None)] onfocus: Option<UnsyncCallback<()>>,
    #[prop(into, default = None)] onblur: Option<UnsyncCallback<()>>,
    #[prop(into, default = None)] onkeydown: Option<UnsyncCallback<KeyboardEvent>>,
    #[prop(into, default = None)] onkeyup: Option<UnsyncCallback<KeyboardEvent>>,
    #[prop(into, default = None)] onenter: Option<UnsyncCallback<()>>,
) -> impl IntoView {
    let on_input = {
        let value = value.clone();
        let onupdate = onupdate.clone();
        move |evt: Event| {
            if let Some(target) = evt.target() {
                let input: HtmlInputElement = target.unchecked_into();
                let new_value = input.value();
                let new_value = if disable_trim {
                    new_value
                } else {
                    new_value.trim().to_string()
                };
                let new_value = SharedString::from(new_value);
                if !readonly {
                    value.set(new_value.clone());
                }
                if let Some(onupdate) = onupdate.as_ref() {
                    onupdate.run(new_value);
                }
                if let Some(oninput) = oninput.as_ref() {
                    oninput.run(evt);
                }
            }
        }
    };
    let on_change = {
        let value = value.clone();
        move |evt: Event| {
            if let Some(target) = evt.target() {
                let input: HtmlInputElement = target.unchecked_into();
                let new_value = input.value();
                let new_value = if disable_trim {
                    new_value
                } else {
                    new_value.trim().to_string()
                };
                let new_value = SharedString::from(new_value);
                if !readonly {
                    value.set(new_value.clone());
                }
                if let Some(onupdate) = onupdate.as_ref() {
                    onupdate.run(new_value);
                }
                if let Some(onchange) = onchange.as_ref() {
                    onchange.run(evt);
                }
            }
        }
    };
    let on_focus = move |_| {
        if let Some(onfocus) = onfocus.as_ref() {
            onfocus.run(());
        }
    };
    let on_blur = move |_| {
        if let Some(onblur) = onblur.as_ref() {
            onblur.run(());
        }
    };
    let on_keydown = move |evt: KeyboardEvent| {
        if let Some(onkeydown) = onkeydown.as_ref() {
            onkeydown.run(evt);
        }
    };
    let on_keyup = move |evt: KeyboardEvent| {
        let key_code = evt.key_code();
        if let Some(onkeyup) = onkeyup.as_ref() {
            onkeyup.run(evt);
        }
        if 13 == key_code {
            if let Some(onenter) = onenter.as_ref() {
                onenter.run(());
            }
        }
    };
    view! {
        <input
            type={r#type}
            class="e-input"
            value={value}
            tabindex={tabindex}
            placeholder={placeholder}
            style={style}
            on:input={on_input}
            on:change={on_change}
            on:focus={on_focus}
            on:blur={on_blur}
            on:keydown={on_keydown}
            on:keyup={on_keyup}
        />
    }
}
