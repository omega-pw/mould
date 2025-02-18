use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Option<Callback<()>>,
    #[prop_or_default]
    pub onmouseenter: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub onmouseleave: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub onmousedown: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub onmouseup: Option<Callback<MouseEvent>>,
    pub children: Children,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    #[prop_or_default]
    pub hover_style: Option<AttrValue>,
    #[prop_or_default]
    pub active_style: Option<AttrValue>,
}

#[function_component]
pub fn Button(props: &Props) -> Html {
    let hover_active: UseStateHandle<bool> = use_state(|| false);
    let press_active: UseStateHandle<bool> = use_state(|| false);
    let disabled = props.disabled;
    let onclick = props.onclick.clone();
    let on_click = Callback::from(move |_evt: MouseEvent| {
        if !disabled {
            if let Some(onclick) = onclick.as_ref() {
                onclick.emit(());
            }
        }
    });
    let on_mouseenter = {
        let hover_active = hover_active.clone();
        let onmouseenter = props.onmouseenter.clone();
        Callback::from(move |evt: MouseEvent| {
            hover_active.set(true);
            if let Some(onmouseenter) = onmouseenter.as_ref() {
                onmouseenter.emit(evt);
            }
        })
    };
    let on_mouseleave = {
        let hover_active = hover_active.clone();
        let onmouseleave = props.onmouseleave.clone();
        Callback::from(move |evt: MouseEvent| {
            hover_active.set(false);
            if let Some(onmouseleave) = onmouseleave.as_ref() {
                onmouseleave.emit(evt);
            }
        })
    };
    let on_mousedown = {
        let press_active = press_active.clone();
        let onmousedown = props.onmousedown.clone();
        Callback::from(move |evt: MouseEvent| {
            press_active.set(true);
            if let Some(onmousedown) = onmousedown.as_ref() {
                onmousedown.emit(evt);
            }
        })
    };
    let on_mouseup = {
        let press_active = press_active.clone();
        let onmouseup = props.onmouseup.clone();
        Callback::from(move |evt: MouseEvent| {
            press_active.set(false);
            if let Some(onmouseup) = onmouseup.as_ref() {
                onmouseup.emit(evt);
            }
        })
    };
    let mut styles = Vec::with_capacity(6);
    if let Some(style) = props.style.as_ref() {
        let style = style.as_str().trim();
        styles.push(style);
        if !style.ends_with(";") {
            styles.push(";");
        }
    }
    if !disabled {
        if let (true, Some(hover_style)) = (*hover_active, props.hover_style.as_ref()) {
            let hover_style = hover_style.as_str().trim();
            styles.push(hover_style);
            if !hover_style.ends_with(";") {
                styles.push(";");
            }
        }
        if let (true, Some(active_style)) = (*press_active, props.active_style.as_ref()) {
            let active_style = active_style.as_str().trim();
            styles.push(active_style);
            if !active_style.ends_with(";") {
                styles.push(";");
            }
        }
    }
    let style: Option<String> = if styles.is_empty() {
        None
    } else {
        Some(styles.concat())
    };
    html! {
        <button
            type="button"
            class="e-btn"
            disabled={disabled}
            onclick={on_click}
            onmouseenter={on_mouseenter}
            onmouseleave={on_mouseleave}
            onmousedown={on_mousedown}
            onmouseup={on_mouseup}
            style={style}
        >
            {props.children.clone()}
        </button>
    }
}
