use super::button::Button;
use super::svg_icon::AngleDown;
use super::svg_icon::AngleLeft;
use super::svg_icon::AngleRight;
use super::svg_icon::AngleUp;
use super::svg_icon::Close;
use super::svg_icon::Question;
use crate::SharedString;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[derive(Clone, PartialEq)]
pub enum Icon {
    Close,
    AngleUp,
    AngleDown,
    AngleLeft,
    AngleRight,
    Question,
}

#[component]
pub fn IconButton(
    icon: Icon,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, default = None)] onclick: Option<UnsyncCallback<()>>,
    #[prop(into, optional)] style: SharedString,
    #[prop(into, optional)] icon_style: SharedString,
    #[prop(into, optional)] color: SharedString,
    #[prop(into, optional)] bgcolor: SharedString,
    #[prop(into, optional)] hover_color: SharedString,
    #[prop(into, optional)] hover_bgcolor: SharedString,
) -> impl IntoView {
    let hover_active: RwSignal<bool> = RwSignal::new(false);
    let on_mouseenter = {
        let hover_active = hover_active.clone();
        UnsyncCallback::new(move |_evt: MouseEvent| {
            hover_active.set(true);
        })
    };
    let on_mouseleave = {
        let hover_active = hover_active.clone();
        UnsyncCallback::new(move |_evt: MouseEvent| {
            hover_active.set(false);
        })
    };
    let color = move || {
        if !disabled.get() && hover_active.get() {
            if !hover_color.is_empty() {
                Some(hover_color.clone())
            } else {
                None
            }
        } else {
            if !color.is_empty() {
                Some(color.clone())
            } else {
                None
            }
        }
    };
    let style = MaybeProp::derive(move || {
        let bgcolor = if !disabled.get() && hover_active.get() {
            if !hover_bgcolor.is_empty() {
                Some(hover_bgcolor.clone())
            } else {
                None
            }
        } else {
            if !bgcolor.is_empty() {
                Some(bgcolor.clone())
            } else {
                None
            }
        };
        Some(SharedString::from(format!(
            "border-style: none;background-color: {};padding-left: 0;padding-right: 0;{}",
            bgcolor
                .as_ref()
                .map(|bgcolor| bgcolor.as_str())
                .unwrap_or("transparent"),
            style.as_str()
        )))
    });
    let icon_style = MaybeProp::derive(move || {
        Some(SharedString::from(format!(
            "width: 1em;vertical-align: middle;{}",
            icon_style.as_str()
        )))
    });
    view! {
        <Button
            disabled={disabled}
            onclick={onclick}
            style={style}
            hover_style=SharedString::from("cursor: pointer;")
            onmouseenter={on_mouseenter}
            onmouseleave={on_mouseleave}
        >
            {
                match icon {
                    Icon::Close => view! {
                        <Close style={icon_style} color={MaybeProp::derive(color)}/>
                    }.into_any(),
                    Icon::AngleUp => view! {
                        <AngleUp style={icon_style} color={MaybeProp::derive(color)}/>
                    }.into_any(),
                    Icon::AngleDown => view! {
                        <AngleDown style={icon_style} color={MaybeProp::derive(color)}/>
                    }.into_any(),
                    Icon::AngleLeft => view! {
                        <AngleLeft style={icon_style} color={MaybeProp::derive(color)}/>
                    }.into_any(),
                    Icon::AngleRight => view! {
                        <AngleRight style={icon_style} color={MaybeProp::derive(color)}/>
                    }.into_any(),
                    Icon::Question => view! {
                        <Question style={icon_style} color={MaybeProp::derive(color)}/>
                    }.into_any()
                }
            }
        </Button>
    }
}
