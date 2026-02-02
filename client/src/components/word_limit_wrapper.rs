use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn WordLimitWrapper(
    length: u32,
    maxlength: u32,
    #[prop(into, optional)] style: SharedString,
    #[prop(into, optional)] limit_style: SharedString,
    children: Children,
) -> impl IntoView {
    let mut actual_style = String::from("position: relative;display: inline-block;");
    if !style.is_empty() {
        actual_style.push_str(&style);
    }
    let limit_words = format!("{}/{}", length, maxlength);
    let mut actual_limit_style = String::from("margin: 0;position: absolute;line-height: 1.25em;bottom: 0.25em;right: 0.5em;text-align: right;");
    if !limit_style.is_empty() {
        actual_limit_style.push_str(&limit_style);
    }
    view! {
        <div style={actual_style}>
            { children() }
            <p style={actual_limit_style}>{limit_words}</p>
        </div>
    }
}
