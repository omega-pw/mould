use super::checkbox::Checkbox;
use super::SelectOption;
use std::ops::Deref;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State<O: SelectOption> {
    value: Vec<O::Value>,
    options: Vec<O>,
}

pub enum Msg<V> {
    ChangeValue(V, bool),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<O: SelectOption>
where
    O: Clone + PartialEq,
    O::Value: Clone + PartialEq,
{
    pub value: Vec<O::Value>,
    pub options: Vec<O>,
    pub onchange: Callback<Vec<O::Value>>,
}

pub struct CheckboxGroup<O: SelectOption>
where
    O: Clone + PartialEq + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    state: State<O>,
}

impl<O> Component for CheckboxGroup<O>
where
    O: Clone + PartialEq + SelectOption + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    type Message = Msg<O::Value>;
    type Properties = Props<O>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            value: props.value.clone(),
            options: props.options.clone(),
        };
        CheckboxGroup { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeValue(value, selected) => {
                if selected {
                    if self.state.value.iter().all(|item| item != &value) {
                        self.state.value.push(value);
                        ctx.props().onchange.emit(self.state.value.clone());
                    }
                } else {
                    self.state.value.retain(|item| item != &value);
                    ctx.props().onchange.emit(self.state.value.clone());
                }
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
            <div class="e-checkbox-group">
                {
                    for self.state.options.iter().map(|option| {
                        let current_value = option.value();
                        let selected = self.state.value.iter().any(|val| {
                            val == &current_value
                        });
                        let on_change = ctx.link().callback(move |selected: bool| {
                            Msg::ChangeValue(current_value.clone(), selected)
                        });
                        html! {
                            <label>
                                <Checkbox value={selected} onchange={on_change}/>
                                {option.label()}
                            </label>
                        }
                    })
                }
            </div>
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps<O: SelectOption>
where
    O: Clone + PartialEq,
    O::Value: Clone + PartialEq + 'static,
{
    pub value: UseStateHandle<Vec<O::Value>>,
    pub options: Vec<O>,
    #[prop_or_default]
    pub onchange: Option<Callback<Vec<O::Value>>>,
}

#[function_component]
pub fn BindingCheckboxGroup<O: SelectOption>(props: &BindingProps<O>) -> Html
where
    O: Clone + PartialEq + SelectOption + 'static,
    O::Value: Clone + PartialEq + 'static,
{
    let value_clone = props.value.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |value: Vec<O::Value>| {
        value_clone.set(value.clone());
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(value);
        }
    });
    return html! {
        <CheckboxGroup<O>
            value={props.value.deref().clone()}
            options={props.options.clone()}
            onchange={on_change}
        />
    };
}
