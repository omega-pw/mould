use leptos::prelude::*;

#[component]
pub fn Page(mask: bool, #[prop(default = 1)] z_index: u64, children: Children) -> impl IntoView {
    let mut style = format!("position: absolute;top: 0;left: 0;bottom: 0;right: 0;width: 100%;height: 100%;overflow: hidden;z-index: {};", z_index);
    if mask {
        style.push_str("background-color:rgba(128,128,128,0.5);");
    }
    view! {
        <div style={style}>
            { children() }
        </div>
    }
}
