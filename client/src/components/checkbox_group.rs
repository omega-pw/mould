use super::checkbox::Checkbox;
use super::SelectOption;
use leptos::prelude::*;
use std::hash::Hash;

#[component]
pub fn CheckboxGroup<O: SelectOption>(
    value: RwSignal<Vec<O::Value>>,
    #[prop(into)] options: Signal<Vec<O>>,
    #[prop(optional)] readonly: bool,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<Vec<O::Value>>>,
) -> impl IntoView
where
    O: Clone + PartialEq + Send + Sync + 'static,
    O::Value: Clone + Eq + Hash + Send + Sync + 'static,
{
    view! {
        <div class="e-checkbox-group">
            <For
                each=move || { options.get().into_iter() }
                key=|option| { option.value() }
                children=move |option| {
                    let curr_value = option.value();
                    let selected = value.read().iter().any(|val| {
                        val == &curr_value
                    });
                    let on_change = {
                        let value = value.clone();
                        UnsyncCallback::new(move |selected: bool| {
                            if readonly {
                                return;
                            }
                            let curr_value = curr_value.clone();
                            if selected {
                                if value.read().iter().all(|item| item != &curr_value) {
                                    value.write().push(curr_value);
                                    if let Some(onchange) = onchange.as_ref() {
                                        onchange.run(value.get());
                                    }
                                }
                            } else {
                                value.write().retain(|item| item != &curr_value);
                                if let Some(onchange) = onchange.as_ref() {
                                    onchange.run(value.get());
                                }
                            }
                        })
                    };
                    view! {
                        <label>
                            <Checkbox value={RwSignal::new(selected)} onchange={on_change}/>
                            {option.label()}
                        </label>
                    }
                }
            />
        </div>
    }
}
