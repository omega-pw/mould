use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub condition: bool,
    pub children: Children,
}

#[function_component]
pub fn If(props: &Props) -> Html {
    if props.condition {
        html! {
            { props.children.clone() }
        }
    } else {
        html! {}
    }
}
