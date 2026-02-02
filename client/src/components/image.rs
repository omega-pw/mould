use crate::SharedString;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn Image(
    #[prop(into, optional)] src: MaybeProp<SharedString>,
    #[prop(into, default = None)] onclick: Option<UnsyncCallback<()>>,
    #[prop(into, optional)] style: SharedString,
) -> impl IntoView {
    let on_click = move |_evt: MouseEvent| {
        if let (Some(onclick), true) = (onclick.as_ref(), src.get().is_some()) {
            onclick.run(());
        }
    };
    view! {
        <img src={move || src.get()} on:click={on_click} style={style}/>
    }
}
