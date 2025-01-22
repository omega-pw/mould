use yew::prelude::*;
use yew::{html, Html};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub style: Option<AttrValue>,
    pub children: Children,
}

#[function_component]
pub fn ButtonGroup(props: &Props) -> Html {
    html! {
        <span class="button-group" style={props.style.clone()}>{ props.children.clone() }</span>
    }
}
