use super::focus_area::FocusArea;
use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn Drawer(
    #[prop(into)] active: Signal<bool>,
    #[prop(into, default = None)] onclickother: Option<UnsyncCallback<()>>,
    #[prop(into, optional)] style: SharedString,
    children: ChildrenFn,
) -> impl IntoView {
    let edit_page_class = {
        let active = active.clone();
        move || {
            let right_page_class = "absolute dock-right dock-right-page";
            let right_page_active_class = "absolute dock-right dock-right-page active";
            if active.get() {
                right_page_active_class
            } else {
                right_page_class
            }
        }
    };
    view! {
        <div class={edit_page_class} style={style}>
            <Show
                when=move || { active.get() }
            >
                <FocusArea clone:children onclickother={onclickother.clone()} style={SharedString::from("width:100%;height:100%;")}>
                    { children() }
                </FocusArea>
            </Show>
        </div>
    }
}
