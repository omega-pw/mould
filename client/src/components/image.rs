use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub src: Option<AttrValue>,
    #[prop_or_default]
    pub onclick: Option<Callback<()>>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
}

#[function_component]
pub fn Image(props: &Props) -> Html {
    let onclick = props.onclick.clone();
    let on_click = Callback::from(move |_evt: MouseEvent| {
        if let Some(onclick) = onclick.as_ref() {
            onclick.emit(());
        }
    });
    html! {
        <img src={props.src.clone()} onclick={on_click} style={props.style.clone()}/>
    }
}
