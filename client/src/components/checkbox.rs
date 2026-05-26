use crate::SharedString;
use leptos::prelude::*;
use web_sys::Event;

#[component]
pub fn Checkbox(
    value: RwSignal<bool>,
    #[prop(optional)] readonly: bool,
    #[prop(into, optional)] label: SharedString,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<bool>>,
) -> impl IntoView {
    let on_change = {
        let value = value.clone();
        move |_evt: Event| {
            let old_value = value.get();
            let new_value = !old_value;
            if !readonly {
                value.set(new_value.clone());
            }
            if let Some(onchange) = onchange.as_ref() {
                onchange.run(new_value);
            }
        }
    };
    if !label.is_empty() {
        view! {
            <label>
                <input type="checkbox" checked={value} on:change={on_change} />
                {label}
            </label>
        }
        .into_any()
    } else {
        view! {
            <input type="checkbox" checked={value} on:change={on_change} />
        }
        .into_any()
    }
}
