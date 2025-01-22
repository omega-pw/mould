use super::SelectOption;
use std::ops::Deref;
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State<O: SelectOption> {
    value: Option<O::Value>,
    options: Vec<O>,
}

pub enum Msg {
    ChangeValue(Event),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<O: SelectOption>
where
    O: Clone + PartialEq,
    O::Value: Clone + PartialEq,
{
    pub value: Option<O::Value>,
    pub options: Vec<O>,
    #[prop_or_default]
    pub clearable: bool,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    pub onchange: Callback<Option<O>>,
}

pub struct Selection<O: SelectOption>
where
    O: Clone + PartialEq + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    state: State<O>,
}

impl<O> Component for Selection<O>
where
    O: Clone + PartialEq + SelectOption + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    type Message = Msg;
    type Properties = Props<O>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            value: props.value.clone(),
            options: props.options.clone(),
        };
        Selection { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeValue(evt) => {
                let select: HtmlSelectElement = evt.target_unchecked_into();
                for (index, item) in self.state.options.iter().enumerate() {
                    if select.value() == index.to_string() {
                        ctx.props().onchange.emit(Some(item.clone()));
                        return true;
                    }
                }
                ctx.props().onchange.emit(None);
            }
        }
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.value = props.value.clone();
        self.state.options = props.options.clone();
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_change = ctx.link().callback(|evt: Event| Msg::ChangeValue(evt));
        let clearable = ctx.props().clearable;
        let not_match = !self.state.options.is_empty()
            && self
                .state
                .options
                .iter()
                .all(|option| Some(option.value()) != self.state.value);
        html! {
            <select class="e-select" onchange={on_change}>
                {
                    if clearable || not_match {
                        let placeholder = ctx.props().placeholder.clone().unwrap_or_else(|| {
                            AttrValue::from("--请选择--")
                        });
                        html! {
                            <option value="" selected={not_match}>{placeholder}</option>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    for self.state.options.iter().enumerate().map(|(index, item)| {
                        let selected = Some(item.value()) == self.state.value;
                        html! {
                            <option value={index.to_string()} selected={selected}>{item.label()}</option>
                        }
                    })
                }
            </select>
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
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    #[prop_or_default]
    pub onchange: Option<Callback<Option<O>>>,
}

#[function_component]
pub fn BindingSelection<O: SelectOption>(props: &BindingProps<O>) -> Html
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
        <Selection<O>
            value={props.value.deref().clone()}
            options={props.options.clone()}
            clearable={props.clearable}
            placeholder={props.placeholder.clone()}
            onchange={on_change}
        />
    };
}
