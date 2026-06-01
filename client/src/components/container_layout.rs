use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn HorizontalLayout(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("container-layout is-horizontal")
    } else {
        SharedString::from(format!("container-layout is-horizontal {}", class))
    };
    view! {
        <div class={actual_class} style={style}>
            { children() }
        </div>
    }
}

#[component]
pub fn VerticalLayout(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("container-layout is-vertical")
    } else {
        SharedString::from(format!("container-layout is-vertical {}", class))
    };
    view! {
        <div class={actual_class} style={style}>
            { children() }
        </div>
    }
}

#[component]
pub fn Main(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("main")
    } else {
        SharedString::from(format!("main {}", class))
    };
    view! {
        <div class={actual_class} style={style}>
            { children() }
        </div>
    }
}

#[component]
pub fn Header(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("header")
    } else {
        SharedString::from(format!("header {}", class))
    };
    view! {
        <header class={actual_class} style={style}>
            { children() }
        </header>
    }
}

#[component]
pub fn Footer(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("footer")
    } else {
        SharedString::from(format!("footer {}", class))
    };
    view! {
        <footer class={actual_class} style={style}>
            { children() }
        </footer>
    }
}

#[component]
pub fn Aside(
    #[prop(into, optional)] class: SharedString,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let actual_class = if class.is_empty() {
        SharedString::from("aside")
    } else {
        SharedString::from(format!("aside {}", class))
    };
    view! {
        <aside class={actual_class} style={style}>
            { children() }
        </aside>
    }
}
