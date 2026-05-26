use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn Page(
    mask: bool,
    #[prop(default = 1)] z_index: u64,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let mask_style = if mask {
        "background-color:rgba(128,128,128,0.5);"
    } else {
        ""
    };
    let style = format!("position: absolute;top: 0;left: 0;bottom: 0;right: 0;width: 100%;height: 100%;overflow: hidden;z-index: {};{}{}", z_index, mask_style, style);
    view! {
        <div style={style}>
            { children() }
        </div>
    }
}
