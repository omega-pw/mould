use crate::SharedString;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn Button(
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, default = None)] onclick: Option<UnsyncCallback<()>>,
    #[prop(into, default = None)] onmouseenter: Option<UnsyncCallback<MouseEvent>>,
    #[prop(into, default = None)] onmouseleave: Option<UnsyncCallback<MouseEvent>>,
    #[prop(into, default = None)] onmousedown: Option<UnsyncCallback<MouseEvent>>,
    #[prop(into, default = None)] onmouseup: Option<UnsyncCallback<MouseEvent>>,
    children: Children,
    #[prop(into, optional)] style: MaybeProp<SharedString>,
    #[prop(into, optional)] hover_style: Option<SharedString>,
    #[prop(into, optional)] active_style: Option<SharedString>,
) -> impl IntoView {
    let hover_active: RwSignal<bool> = RwSignal::new(false);
    let press_active: RwSignal<bool> = RwSignal::new(false);
    let on_click = move |_evt: MouseEvent| {
        if !disabled.get() {
            if let Some(onclick) = onclick.as_ref() {
                onclick.run(());
            }
        }
    };
    let on_mouseenter = {
        let hover_active = hover_active.clone();
        move |evt: MouseEvent| {
            hover_active.set(true);
            if let Some(onmouseenter) = onmouseenter.as_ref() {
                onmouseenter.run(evt);
            }
        }
    };
    let on_mouseleave = {
        let hover_active = hover_active.clone();
        move |evt: MouseEvent| {
            hover_active.set(false);
            if let Some(onmouseleave) = onmouseleave.as_ref() {
                onmouseleave.run(evt);
            }
        }
    };
    let on_mousedown = {
        let press_active = press_active.clone();
        move |evt: MouseEvent| {
            press_active.set(true);
            if let Some(onmousedown) = onmousedown.as_ref() {
                onmousedown.run(evt);
            }
        }
    };
    let on_mouseup = {
        let press_active = press_active.clone();
        move |evt: MouseEvent| {
            press_active.set(false);
            if let Some(onmouseup) = onmouseup.as_ref() {
                onmouseup.run(evt);
            }
        }
    };
    let style = move || {
        let mut styles = Vec::with_capacity(6);
        let style = style.get();
        if let Some(style) = style.as_ref() {
            let style = style.as_str().trim();
            styles.push(style);
            if !style.ends_with(";") {
                styles.push(";");
            }
        }
        if !disabled.get() {
            if let (true, Some(hover_style)) = (hover_active.get(), hover_style.as_ref()) {
                let hover_style = hover_style.as_str().trim();
                styles.push(hover_style);
                if !hover_style.ends_with(";") {
                    styles.push(";");
                }
            }
            if let (true, Some(active_style)) = (press_active.get(), active_style.as_ref()) {
                let active_style = active_style.as_str().trim();
                styles.push(active_style);
                if !active_style.ends_with(";") {
                    styles.push(";");
                }
            }
        }
        if styles.is_empty() {
            None
        } else {
            Some(styles.concat())
        }
    };
    view! {
        <button
            type="button"
            class="e-btn"
            disabled={disabled}
            on:click={on_click}
            on:mouseenter={on_mouseenter}
            on:mouseleave={on_mouseleave}
            on:mousedown={on_mousedown}
            on:mouseup={on_mouseup}
            style={style}
        >
            {children()}
        </button>
    }
}
