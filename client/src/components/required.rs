use leptos::prelude::*;

#[component]
pub fn Required() -> impl IntoView {
    view! {
        <span style="color:red;margin-right: 0.25em;">{"*"}</span>
    }
}
