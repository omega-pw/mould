use super::SelectOption;
use crate::SharedString;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlSelectElement};

#[component]
pub fn Selection<O: SelectOption>(
    value: RwSignal<Option<O::Value>>,
    #[prop(into)] options: Signal<Vec<O>>,
    #[prop(optional)] readonly: bool,
    #[prop(optional)] clearable: bool,
    #[prop(into, default = None)] placeholder: Option<SharedString>,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<Option<O>>>,
) -> impl IntoView
where
    O: Clone + PartialEq + Send + Sync + 'static,
    O::Value: Clone + PartialEq + Send + Sync + 'static,
{
    let on_change = {
        let value = value.clone();
        let options = options.clone();
        move |evt: Event| {
            if let Some(target) = evt.target() {
                let select: HtmlSelectElement = target.unchecked_into();
                let mut new_value = None;
                for (index, item) in options.read().iter().enumerate() {
                    if select.value() == index.to_string() {
                        new_value.replace(item.clone());
                        break;
                    }
                }
                if !readonly {
                    value.set(new_value.as_ref().map(|new_value| new_value.value()));
                }
                if let Some(onchange) = onchange.as_ref() {
                    onchange.run(new_value);
                }
            }
        }
    };
    let not_match = {
        let value = value.clone();
        let options = options.clone();
        move || {
            let value = value.read();
            let options = options.read();
            !options.is_empty() && options.iter().all(|option| value != Some(option.value()))
        }
    };
    view! {
        <select class="e-select" on:change={on_change}>
            <Show
                when={
                    let not_match = not_match.clone();
                    move || { clearable || not_match() }
                }
            >
                {
                    let placeholder = placeholder.clone().unwrap_or_else(|| {
                        SharedString::from("--请选择--")
                    });
                    view! {
                        <option value="" selected={not_match.clone()}>{placeholder}</option>
                    }
                }
            </Show>
            <For
                each=move || { options.get().into_iter().enumerate() }
                key=|(index, _option)| { *index }
                children=move |(index, option)| {
                    let selected = {
                        let value = value.clone();
                        let curr_value = option.value();
                        move || {
                            value.read().as_ref() == Some(&curr_value)
                        }
                    };
                    view! {
                        <option value={index.to_string()} selected={selected}>{option.label()}</option>
                    }
                }
            />
        </select>
    }
}
