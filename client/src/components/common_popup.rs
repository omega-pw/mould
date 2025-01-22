use yew::prelude::*;
use yew::{html, Html};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub active: bool,
    pub z_index: Option<i32>,
    pub children: Children,
}

#[function_component]
pub fn CommonPopup(props: &Props) -> Html {
    let mut style = None;
    if let Some(z_index) = props.z_index {
        style.replace(format!("z-index: {}", z_index));
    }
    let class = if props.active {
        "popup-content active"
    } else {
        "popup-content"
    };
    html! {
        <div class="popup-root" style={style}>
            <div class={class}>
                { props.children.clone() }
            </div>
        </div>
    }
}
