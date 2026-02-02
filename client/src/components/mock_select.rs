use super::on_cleanup_unsync;
use super::SelectOption;
use crate::SharedString;
use js_sys::Function;
use leptos::prelude::*;
use std::hash::Hash;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlInputElement, MouseEvent};

#[component]
pub fn MockSelect<O: SelectOption>(
    value: RwSignal<Option<O::Value>>,
    #[prop(into)] options: Signal<Vec<O>>,
    #[prop(optional)] readonly: bool,
    #[prop(optional)] clearable: bool,
    #[prop(into, default = SharedString::from("请选择"))] placeholder: SharedString,
    #[prop(optional)] searchable: bool,
    #[prop(into, default = SharedString::from("搜索"))] search_placeholder: SharedString,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<Option<O>>>,
    #[prop(into, default = None)] onsearch: Option<UnsyncCallback<Option<SharedString>>>,
) -> impl IntoView
where
    O: Clone + PartialEq + Send + Sync + 'static,
    O::Value: Clone + Eq + Hash + Send + Sync + 'static,
{
    let panel_active = RwSignal::new(false);
    let is_clear = RwSignal::new(false);
    let document = web_sys::window().unwrap().document().unwrap();
    let listener: Function = Closure::wrap(Box::new({
        let panel_active = panel_active.clone();
        move || {
            panel_active.set(false);
        }
    }) as Box<dyn Fn()>)
    .into_js_value()
    .dyn_into()
    .unwrap();
    document
        .add_event_listener_with_callback("click", &listener)
        .unwrap();
    on_cleanup_unsync(move || {
        document
            .remove_event_listener_with_callback("click", &listener)
            .unwrap();
    });

    let on_root_click = |evt: MouseEvent| {
        evt.stop_propagation();
    };
    let on_search = {
        let onsearch = onsearch.clone();
        move |evt: Event| {
            if let Some(target) = evt.target() {
                let input: HtmlInputElement = target.unchecked_into();
                if let Some(onsearch) = onsearch.as_ref() {
                    let word = input.value();
                    let word = word.trim();
                    onsearch.run(if word.is_empty() {
                        None
                    } else {
                        Some(SharedString::from(word.to_string()))
                    });
                }
            }
        }
    };
    let on_open_panel = {
        let onsearch = onsearch.clone();
        let panel_active = panel_active.clone();
        move |_| {
            panel_active.set(true);
            if let Some(onsearch) = onsearch.as_ref() {
                onsearch.run(None);
            }
        }
    };
    let active_label_opt = {
        let value = value.clone();
        let options = options.clone();
        move || {
            let mut label_opt: Option<AnyView> = None;
            if let Some(value) = value.read().as_ref() {
                for option in options.read().iter() {
                    if &option.value() == value {
                        label_opt = Some(option.label());
                        break;
                    }
                }
            }
            label_opt
        }
    };
    let label = {
        let active_label_opt = active_label_opt.clone();
        move || active_label_opt().unwrap_or(placeholder.clone().into_any())
    };
    let box_class = move || {
        let mut class = String::from("e-mock-select-box");
        if active_label_opt().is_none() {
            class.push_str(" s-empty");
        }
        class
    };
    let on_clear = {
        let value = value.clone();
        let panel_active = panel_active.clone();
        let onchange = onchange.clone();
        move |evt: MouseEvent| {
            evt.stop_propagation();
            panel_active.set(false);
            if !readonly {
                value.set(None);
            }
            if let Some(onchange) = onchange.as_ref() {
                onchange.run(None);
            }
        }
    };
    let on_mouseenter = {
        let value = value.clone();
        let is_clear = is_clear.clone();
        move |_evt: MouseEvent| {
            let mut new_is_clear = true;
            if !clearable || value.read().is_none() {
                new_is_clear = false;
            }
            is_clear.set(new_is_clear);
        }
    };
    let on_mouseleave = {
        let is_clear = is_clear.clone();
        move |_evt: MouseEvent| {
            is_clear.set(false);
        }
    };
    let search_placeholder = StoredValue::new(search_placeholder);
    view! {
        <div class="e-mock-select" on:click={on_root_click} style="position:relative;">
            <div class={box_class} on:click={on_open_panel} style="padding: 0.25em;border-width: 1px;border-style: solid;">
                {label}
                <span style="float:right;" on:mouseenter={on_mouseenter} on:mouseleave={on_mouseleave}>
                    {
                        let is_clear = is_clear.clone();
                        move || {
                            if is_clear.get() {
                                view! {
                                    <i class="fa fa-times" aria-hidden="true" on:click={on_clear.clone()} style="line-height: normal;cursor: pointer;"></i>
                                }.into_any()
                            } else {
                                view! {
                                    <i class="fa fa-caret-down" aria-hidden="true" style="line-height: normal;"></i>
                                }.into_any()
                            }
                        }
                    }
                </span>
            </div>
            <Show
                when={
                    let panel_active = panel_active.clone();
                    move || { panel_active.get() }
                }
            >
                <div class="e-mock-option-panel" style="position:absolute;left:0;right:0;top:100%;border-left-width: 1px;border-left-style: solid;border-right-width: 1px;border-right-style: solid;border-bottom-width: 1px;border-bottom-style: solid;">
                    <Show
                        when=move || { searchable }
                    >
                        <div style="padding:0.25em;">
                            <input type="text" class="e-mock-search-input" on:input={on_search} placeholder={search_placeholder.read_value().clone()} style="box-sizing: border-box;width: 100%;padding-top: 0.25em;padding-bottom: 0.25em;"/>
                        </div>
                    </Show>
                    <ul style="margin: 0;padding: 0;list-style-type: none;max-height: 20em;overflow-y: auto;">
                        <For
                            each={
                                let options = options.clone();
                                move || { options.get().into_iter() }
                            }
                            key=|option| { option.value() }
                            children={
                                let value = value.clone();
                                let panel_active = panel_active.clone();
                                let onchange = onchange.clone();
                                move |option| {
                                    let value = value.clone();
                                    let panel_active = panel_active.clone();
                                    let onchange = onchange.clone();
                                    let on_change = {
                                        let option = option.clone();
                                        move |_| {
                                            panel_active.set(false);
                                            if !readonly {
                                                value.set(Some(option.value()));
                                            }
                                            if let Some(onchange) = onchange.as_ref() {
                                                onchange.run(Some(option.clone()));
                                            }
                                        }
                                    };
                                    view! {
                                        <li class="e-mock-option" on:click={on_change} style="padding: 0.25em;">{option.label()}</li>
                                    }
                                }
                            }
                        />
                    </ul>
                </div>
            </Show>
        </div>
    }
}
