use yew::prelude::*;

#[function_component]
pub fn Required() -> Html {
    html! {
        <span style="color:red;margin-right: 0.25em;">{"*"}</span>
    }
}
