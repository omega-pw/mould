use super::SelectOption;
use crate::LightString;
use js_sys::Function;
use std::ops::Deref;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State<O: SelectOption> {
    value: Option<O::Value>,
    options: Vec<O>,
    placeholder: LightString,
    search_placeholder: LightString,
    searchable: bool,
    panel_active: bool,
    is_clear: bool,
}

pub enum Msg<O> {
    ChangeValue(Option<O>),
    Search(String),
    ChangePanelState(bool),
    ChangeClearState(bool),
    Noop,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<O: SelectOption>
where
    O: Clone + PartialEq,
    O::Value: PartialEq + Clone,
{
    pub value: Option<O::Value>,
    pub options: Vec<O>,
    #[prop_or_default]
    pub clearable: bool,
    #[prop_or_else(||LightString::from("请选择"))]
    pub placeholder: LightString,
    #[prop_or_default]
    pub searchable: bool,
    #[prop_or_else(||LightString::from("搜索"))]
    pub search_placeholder: LightString,
    pub onchange: Callback<Option<O>>,
    #[prop_or_default]
    pub onsearch: Option<Callback<Option<LightString>>>,
}

pub struct MockSelect<O: SelectOption>
where
    O: Clone + PartialEq + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    state: State<O>,
    clear_tasks: Vec<Box<dyn Fn()>>,
}

impl<O> Component for MockSelect<O>
where
    O: Clone + PartialEq + SelectOption + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    type Message = Msg<O>;
    type Properties = Props<O>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            value: props.value.clone(),
            options: props.options.clone(),
            placeholder: props.placeholder.clone(),
            search_placeholder: props.search_placeholder.clone(),
            searchable: props.searchable,
            panel_active: false,
            is_clear: false,
        };
        let mut clear_tasks: Vec<Box<dyn Fn()>> = Vec::new();
        let document = web_sys::window().unwrap().document().unwrap();
        let link = ctx.link().clone();
        let listener: Function = Closure::wrap(Box::new(move || {
            link.send_message(Msg::ChangePanelState(false));
        }) as Box<dyn Fn()>)
        .into_js_value()
        .dyn_into()
        .unwrap();
        document
            .add_event_listener_with_callback("click", &listener)
            .unwrap();
        clear_tasks.push(Box::new(move || {
            document
                .remove_event_listener_with_callback("click", &listener)
                .unwrap();
        }));
        MockSelect {
            state,
            clear_tasks: clear_tasks,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeValue(val) => {
                self.state.panel_active = false;
                ctx.props().onchange.emit(val);
            }
            Msg::Search(word) => {
                if let Some(onsearch) = ctx.props().onsearch.as_ref() {
                    let word = word.trim();
                    onsearch.emit(if word.is_empty() {
                        None
                    } else {
                        Some(LightString::from(word.to_string()))
                    });
                }
            }
            Msg::ChangePanelState(panel_active) => {
                self.state.panel_active = panel_active;
                if self.state.panel_active {
                    if let Some(onsearch) = ctx.props().onsearch.as_ref() {
                        onsearch.emit(None);
                    }
                }
            }
            Msg::ChangeClearState(mut is_clear) => {
                if is_clear && (!ctx.props().clearable || self.state.value.is_none()) {
                    is_clear = false;
                }
                if is_clear == self.state.is_clear {
                    return false;
                }
                self.state.is_clear = is_clear;
            }
            Msg::Noop => (),
        }
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.value = props.value.clone();
        self.state.options = props.options.clone();
        self.state.placeholder = props.placeholder.clone();
        self.state.search_placeholder = props.search_placeholder.clone();
        self.state.searchable = props.searchable;
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let on_root_click = link.callback(|evt: MouseEvent| {
            evt.stop_propagation();
            Msg::Noop
        });
        let on_search = link.callback(|evt: InputEvent| {
            let input: HtmlInputElement = evt.target_unchecked_into();
            Msg::Search(input.value().trim().to_string())
        });
        let on_open_panel = link.callback(|_| Msg::ChangePanelState(true));
        let mut box_class = String::from("e-mock-select-box");

        let mut label_opt: Option<Html> = None;
        match self.state.value.as_ref() {
            Some(val) => {
                for option in &self.state.options {
                    if &option.value() == val {
                        label_opt = Some(option.label());
                        break;
                    }
                }
            }
            None => (),
        }
        let label: Html = label_opt.unwrap_or_else(|| {
            box_class.push_str(" s-empty");
            self.state.placeholder.clone().into()
        });
        let on_clear = link.callback(|evt: MouseEvent| {
            evt.stop_propagation();
            Msg::ChangeValue(None)
        });
        let on_mouseenter = link.callback(|_evt: MouseEvent| Msg::ChangeClearState(true));
        let on_mouseleave = link.callback(|_evt: MouseEvent| Msg::ChangeClearState(false));
        html! {
            <div class="e-mock-select" onclick={on_root_click} style="position:relative;">
                <div class={box_class} onclick={on_open_panel} style="padding: 0.25em;border-width: 1px;border-style: solid;">
                    {label}
                    <span style="float:right;" onmouseenter={on_mouseenter} onmouseleave={on_mouseleave}>
                        {
                            if self.state.is_clear {
                                html! {
                                    <i class="fa fa-times" aria-hidden="true" onclick={on_clear} style="line-height: normal;cursor: pointer;"></i>
                                }
                            } else {
                                html! {
                                    <i class="fa fa-caret-down" aria-hidden="true" style="line-height: normal;"></i>
                                }
                            }
                        }
                    </span>
                </div>
                {
                    if self.state.panel_active {
                        html! {
                            <div class="e-mock-option-panel" style="position:absolute;left:0;right:0;top:100%;border-left-width: 1px;border-left-style: solid;border-right-width: 1px;border-right-style: solid;border-bottom-width: 1px;border-bottom-style: solid;">
                                {
                                    if self.state.searchable {
                                        html! {
                                            <div style="padding:0.25em;">
                                                <input type="text" class="e-mock-search-input" oninput={on_search} placeholder={self.state.search_placeholder.clone()} style="box-sizing: border-box;width: 100%;padding-top: 0.25em;padding-bottom: 0.25em;"/>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                                <ul style="margin: 0;padding: 0;list-style-type: none;max-height: 20em;overflow-y: auto;">
                                    {
                                        for self.state.options.iter().map(|item| {
                                            let val = item.clone();
                                            let on_change = link.callback(move |_| Msg::ChangeValue(Some(val.clone())));
                                            html! {
                                                <li class="e-mock-option" onclick={on_change} style="padding: 0.25em;">{item.label()}</li>
                                            }
                                        })
                                    }
                                </ul>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        while let Some(clear_task) = self.clear_tasks.pop() {
            clear_task();
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps<O>
where
    O: Clone + PartialEq + SelectOption,
    O::Value: PartialEq,
{
    pub value: UseStateHandle<Option<O::Value>>,
    pub options: Vec<O>,
    #[prop_or_default]
    pub clearable: bool,
    #[prop_or_else(||LightString::from("请选择"))]
    pub placeholder: LightString,
    #[prop_or_default]
    pub searchable: bool,
    #[prop_or_else(||LightString::from("搜索"))]
    pub search_placeholder: LightString,
    #[prop_or_default]
    pub onchange: Option<Callback<Option<O>>>,
    #[prop_or_default]
    pub onsearch: Option<Callback<Option<LightString>>>,
}

#[function_component]
pub fn BindingMockSelect<O: SelectOption>(props: &BindingProps<O>) -> Html
where
    O: Clone + PartialEq + SelectOption + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    let value_clone = props.value.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |option: Option<O>| {
        value_clone.set(option.as_ref().map(|option| option.value()));
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(option);
        }
    });
    return html! {
        <MockSelect<O>
            value={props.value.deref().clone()}
            options={props.options.clone()}
            clearable={props.clearable}
            placeholder={props.placeholder.clone()}
            searchable={props.searchable}
            search_placeholder={props.search_placeholder.clone()}
            onchange={on_change}
            onsearch={props.onsearch.clone()}
        />
    };
}
