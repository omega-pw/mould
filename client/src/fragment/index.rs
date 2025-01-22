use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

#[function_component]
pub fn Index(_props: &Props) -> Html {
    html! {
        <div class="relative width-fill height-fill">
            {"welcome"}
        </div>
    }
}
