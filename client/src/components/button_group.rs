use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn ButtonGroup(
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    view! {
        <span class="button-group" style={style}>{ children() }</span>
    }
}
