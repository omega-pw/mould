use super::icon_button::Icon;
use super::icon_button::IconButton;
use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn Dialog(
    #[prop(into)] title: Signal<SharedString>,
    closable: bool,
    #[prop(into, optional)] style: SharedString,
    #[prop(into, optional)] content_style: SharedString,
    #[prop(into, default = None)] onclose: Option<UnsyncCallback<()>>,
    children: Children,
) -> impl IntoView {
    let mut actual_content_style = String::from("background-color:#FFF;");
    if !content_style.is_empty() {
        actual_content_style.push_str(&content_style);
    }
    view! {
        <div style={style.to_string()}>
            <div class="e-title-bar" style="height: 2em;line-height: 2em;position: relative;font-weight: normal;margin: 0;padding-left: 0.5em;">
                {title}
                <Show
                    when=move || { closable }
                >
                    <IconButton icon=Icon::Close onclick={onclose} color="#fff" style="position: absolute;top: 0;right: 0;width: 2em;height: 2em;text-align: center;"/>
                </Show>
            </div>
            <div style={actual_content_style}>
                { children() }
            </div>
        </div>
    }
}
