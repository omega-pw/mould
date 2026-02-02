use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn Visable(
    #[prop(into)] condition: Signal<bool>,
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let style = move || {
        let condition = condition.get();
        if style.is_empty() {
            if condition {
                None
            } else {
                Some(SharedString::from("display:none"))
            }
        } else {
            if condition {
                Some(style.clone())
            } else {
                Some(SharedString::from(format!("{};display:none", style)))
            }
        }
    };
    view! {
        <div class={class} style={style}>
            { children() }
        </div>
    }
}
