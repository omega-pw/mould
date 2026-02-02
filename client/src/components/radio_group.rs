use super::SelectOption;
use leptos::prelude::*;
use std::hash::Hash;

#[component]
pub fn RadioGroup<O: SelectOption>(
    value: RwSignal<Option<O::Value>>,
    #[prop(into)] options: Signal<Vec<O>>,
    #[prop(optional)] readonly: bool,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<O>>,
) -> impl IntoView
where
    O: Clone + PartialEq + Send + Sync + 'static,
    O::Value: Clone + Eq + Hash + Send + Sync + 'static,
{
    view! {
        <span>
            <For
                each=move || { options.get().into_iter() }
                key=|option| { option.value() }
                children=move |option| {
                    let on_click = {
                        let value = value.clone();
                        let option = option.clone();
                        move |_| {
                            if !readonly {
                                value.set(Some(option.value()));
                            }
                            if let Some(onchange) = onchange.as_ref() {
                                onchange.run(option.clone());
                            }
                        }
                    };
                    let checked = {
                        let value = value.clone();
                        let curr_value = option.value();
                        move || {
                            value.read().as_ref() == Some(&curr_value)
                        }
                    };
                    view! {
                        <label class="e-radio-label">
                            <input type="radio" checked={checked} on:click={on_click} />
                            {option.label()}
                        </label>
                    }
                }
            />
        </span>
    }
}
