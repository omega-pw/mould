use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn FrameLayout(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("frame-layout")
    } else {
        SharedString::from(format!("frame-layout {}", class))
    };
    view! {
        <div class={actual_class} style={style}>
            { children() }
        </div>
    }
}

#[component]
pub fn Frame(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("frame")
    } else {
        SharedString::from(format!("frame {}", class))
    };
    view! {
        <div class={actual_class} style={style}>
            { children() }
        </div>
    }
}
