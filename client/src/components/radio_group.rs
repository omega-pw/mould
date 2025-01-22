use super::SelectOption;
use std::ops::Deref;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State<O: SelectOption> {
    value: Option<O::Value>,
    options: Vec<O>,
}

pub enum Msg<O> {
    ChangeValue(O),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<O: SelectOption>
where
    O: Clone + PartialEq,
    O::Value: Clone + PartialEq,
{
    pub value: Option<O::Value>,
    pub options: Vec<O>,
    pub onchange: Callback<O>,
}

pub struct RadioGroup<O: SelectOption>
where
    O: Clone + PartialEq + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    state: State<O>,
}

impl<O> Component for RadioGroup<O>
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
        };
        RadioGroup { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeValue(val) => {
                ctx.props().onchange.emit(val);
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
        html! {
            <span>
                {
                    for self.state.options.iter().map(|item| {
                        let val = item.clone();
                        let on_click = ctx.link().callback(move |_| Msg::ChangeValue(val.clone()));
                        let checked = match self.state.value.as_ref() {
                            Some(val) => &item.value() == val,
                            None => false
                        };
                        html! {
                            <label class="e-radio-label">
                                <input type="radio" checked={checked} onclick={on_click} />
                                {item.label()}
                            </label>
                        }
                    })
                }
            </span>
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps<O, VOpt>
where
    O: Clone + PartialEq,
    VOpt: Clone + PartialEq,
{
    pub value: UseStateHandle<VOpt>,
    pub options: Vec<O>,
    #[prop_or_default]
    pub onchange: Option<Callback<O>>,
}

#[function_component]
pub fn BindingRadioGroup<O: SelectOption, VOpt = <O as SelectOption>::Value>(
    props: &BindingProps<O, VOpt>,
) -> Html
where
    O: Clone + PartialEq + 'static,
    O::Value: Clone + PartialEq + 'static,
    VOpt: Clone + PartialEq + From<O::Value> + Into<Option<O::Value>> + 'static,
{
    let value_clone = props.value.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |option: O| {
        value_clone.set(option.value().into());
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(option);
        }
    });
    return html! {
        <RadioGroup<O>
            value={props.value.deref().clone().into()}
            options={props.options.clone()}
            onchange={on_change}
        />
    };
}
