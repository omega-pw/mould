use leptos::prelude::*;

#[component]
pub fn CommonPopup(
    #[prop(into)] active: Signal<bool>,
    #[prop(into, default = None)] z_index: Option<i32>,
    children: Children,
) -> impl IntoView {
    let mut style = None;
    if let Some(z_index) = z_index {
        style.replace(format!("z-index: {}", z_index));
    }
    let class = move || {
        if active.get() {
            "popup-content active"
        } else {
            "popup-content"
        }
    };
    view! {
        <div class="popup-root" style={style}>
            <div class={class}>
                { children() }
            </div>
        </div>
    }
}
