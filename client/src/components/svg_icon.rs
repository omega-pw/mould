use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn Close(
    #[prop(into, optional)] color: MaybeProp<SharedString>,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 384 512" style={move || style.get()}>
            <path fill={move || color.get()} d="M342.6 150.6c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0L192 210.7 86.6 105.4c-12.5-12.5-32.8-12.5-45.3 0s-12.5 32.8 0 45.3L146.7 256 41.4 361.4c-12.5 12.5-12.5 32.8 0 45.3s32.8 12.5 45.3 0L192 301.3 297.4 406.6c12.5 12.5 32.8 12.5 45.3 0s12.5-32.8 0-45.3L237.3 256 342.6 150.6z"/>
        </svg>
    }
}

#[component]
pub fn AngleUp(
    #[prop(into, optional)] color: MaybeProp<SharedString>,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512" style={move || style.get()}>
            <path fill={move || color.get()} d="M201.4 137.4c12.5-12.5 32.8-12.5 45.3 0l160 160c12.5 12.5 12.5 32.8 0 45.3s-32.8 12.5-45.3 0L224 205.3 86.6 342.6c-12.5 12.5-32.8 12.5-45.3 0s-12.5-32.8 0-45.3l160-160z"/>
        </svg>
    }
}

#[component]
pub fn AngleDown(
    #[prop(into, optional)] color: MaybeProp<SharedString>,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512" style={move || style.get()}>
            <path fill={move || color.get()} d="M137.4 374.6c12.5 12.5 32.8 12.5 45.3 0l128-128c9.2-9.2 11.9-22.9 6.9-34.9s-16.6-19.8-29.6-19.8L32 192c-12.9 0-24.6 7.8-29.6 19.8s-2.2 25.7 6.9 34.9l128 128z"/>
        </svg>
    }
}

#[component]
pub fn AngleLeft(
    #[prop(into, optional)] color: MaybeProp<SharedString>,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512" style={move || style.get()}>
            <path fill={move || color.get()} d="M41.4 233.4c-12.5 12.5-12.5 32.8 0 45.3l160 160c12.5 12.5 32.8 12.5 45.3 0s12.5-32.8 0-45.3L109.3 256 246.6 118.6c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0l-160 160z"/>
        </svg>
    }
}

#[component]
pub fn AngleRight(
    #[prop(into, optional)] color: MaybeProp<SharedString>,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512" style={move || style.get()}>
            <path fill={move || color.get()} d="M278.6 233.4c12.5 12.5 12.5 32.8 0 45.3l-160 160c-12.5 12.5-32.8 12.5-45.3 0s-12.5-32.8 0-45.3L210.7 256 73.4 118.6c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0l160 160z"/>
        </svg>
    }
}

#[component]
pub fn Check(
    #[prop(into, optional)] color: MaybeProp<SharedString>,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512" style={move || style.get()}>
            <path fill={move || color.get()} d="M438.6 105.4c12.5 12.5 12.5 32.8 0 45.3l-256 256c-12.5 12.5-32.8 12.5-45.3 0l-128-128c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0L160 338.7 393.4 105.4c12.5-12.5 32.8-12.5 45.3 0z"/>
        </svg>
    }
}

#[component]
pub fn Question(
    #[prop(into, optional)] color: MaybeProp<SharedString>,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" style={move || style.get()}>
            <path fill={move || color.get()} d="M464 256A208 208 0 1 0 48 256a208 208 0 1 0 416 0zM0 256a256 256 0 1 1 512 0A256 256 0 1 1 0 256zm169.8-90.7c7.9-22.3 29.1-37.3 52.8-37.3h58.3c34.9 0 63.1 28.3 63.1 63.1c0 22.6-12.1 43.5-31.7 54.8L280 264.4c-.2 13-10.9 23.6-24 23.6c-13.3 0-24-10.7-24-24V250.5c0-8.6 4.6-16.5 12.1-20.8l44.3-25.4c4.7-2.7 7.6-7.7 7.6-13.1c0-8.4-6.8-15.1-15.1-15.1H222.6c-3.4 0-6.4 2.1-7.5 5.3l-.4 1.2c-4.4 12.5-18.2 19-30.6 14.6s-19-18.2-14.6-30.6l.4-1.2zM224 352a32 32 0 1 1 64 0 32 32 0 1 1 -64 0z"/>
        </svg>
    }
}
