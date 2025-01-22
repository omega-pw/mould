use super::tree::Node;
use super::tree::TreeNode;
use std::ops::Deref;
use std::sync::Arc;
use yew::prelude::*;
use yew::virtual_dom::Key;
use yew_router::prelude::use_navigator;
use yew_router::AnyRoute;
use yew_router::Routable;

#[derive(Clone, PartialEq)]
pub struct State {
    pub expanded_key: UseStateHandle<Option<Key>>,
    pub route: UseStateHandle<Option<String>>,
}

#[derive(Clone, PartialEq)]
pub struct MenuNode {
    pub state: Arc<State>,
    pub key: Key,
    pub name: String,
    pub route: Option<AnyRoute>,
    pub children: Option<Arc<Vec<MenuNode>>>,
}

impl Node for MenuNode {
    fn key(&self) -> Key {
        self.key.clone()
    }
    fn render(&self) -> Html {
        let is_active = if let (Some(node_route), Some(current_route)) =
            (self.state.route.as_ref(), self.route.as_ref())
        {
            node_route == &current_route.to_path()
        } else {
            false
        };
        let addon_style = if is_active {
            "background-color: #EEE;"
        } else {
            ""
        };
        let style = format!(
            "cursor: default;padding-top: 0.5em;padding-bottom: 0.5em;padding-left: 0.5em;{}",
            addon_style
        );
        html! {
            <div style={style}>{self.name.clone()}</div>
        }
    }
    fn children(&self) -> Option<&[Self]> {
        self.children.as_ref().map(|children| children.as_slice())
    }
    fn children_style(&self) -> Option<AttrValue> {
        let base_style = "margin: 0;padding: 0;list-style-type: none;";
        let addon_style = if &Some(self.key.clone()) == self.state.expanded_key.deref() {
            "height: auto;"
        } else {
            "height: 0;overflow: hidden;"
        };
        Some(format!("{}{}", base_style, addon_style).into())
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub list: Vec<MenuNode>,
}

#[function_component]
pub fn Menu(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let ul_style = "margin: 0;padding: 0;list-style-type: none;";
    html! {
        <ul style={ul_style}>
            {
                for props.list.iter().map(|data| {
                    let navigator = navigator.clone();
                    let onclick = Callback::from(move |data: MenuNode| {
                        if data.children.is_none() {
                            if let Some(route) = data.route.as_ref() {
                                navigator.push(route);
                            }
                        } else {
                            let expanded_key = &data.state.expanded_key;
                            if &Some(data.key.clone()) == expanded_key.deref() {
                                expanded_key.set(None);
                            } else {
                                expanded_key.set(Some(data.key.clone()));
                            }
                        }
                    });
                    html! {
                        <TreeNode<MenuNode>
                            key={data.key()}
                            data={data.clone()}
                            onclick={onclick}
                        />
                    }
                })
            }
        </ul>
    }
}
