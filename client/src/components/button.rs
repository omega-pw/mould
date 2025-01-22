use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Option<Callback<()>>,
    pub children: Children,
    #[prop_or_default]
    pub style: Option<AttrValue>,
}

#[function_component]
pub fn Button(props: &Props) -> Html {
    let disabled = props.disabled;
    let onclick = props.onclick.clone();
    let on_click = Callback::from(move |_evt: MouseEvent| {
        if !disabled {
            if let Some(onclick) = onclick.as_ref() {
                onclick.emit(());
            }
        }
    });
    html! {
        <button type="button" class="e-btn" disabled={disabled} onclick={on_click} style={props.style.clone()}>
            {props.children.clone()}
        </button>
    }
}
