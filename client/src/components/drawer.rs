use super::focus_area::FocusArea;
use super::r#if::If;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub active: bool,
    #[prop_or_default]
    pub onclickother: Option<Callback<()>>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    pub children: Children,
}

#[function_component]
pub fn Drawer(props: &Props) -> Html {
    let right_page_class = "absolute dock-right dock-right-page";
    let right_page_active_class = "absolute dock-right dock-right-page active";
    let edit_page_class = if props.active {
        right_page_active_class
    } else {
        right_page_class
    };
    html! {
        <div class={edit_page_class}>
            <If condition={props.active}>
                <FocusArea onclickother={props.onclickother.clone()} style="width:100%;height:100%;">
                    { props.children.clone() }
                </FocusArea>
            </If>
        </div>
    }
}
