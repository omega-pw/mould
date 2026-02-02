use super::tree::Node;
use super::tree::TreeNode;
use crate::Key;
use crate::SharedString;
use leptos::prelude::*;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone)]
pub struct State {
    pub expanded_key: RwSignal<Option<Key>>,
    pub active_key: RwSignal<Option<Key>>,
}

#[derive(Clone)]
pub struct MenuNode {
    pub state: Arc<State>,
    pub key: Key,
    pub name: String,
    pub action: Option<UnsyncCallback<Key>>,
    pub children: Option<Arc<Vec<MenuNode>>>,
}

impl Node for MenuNode {
    fn key(&self) -> Key {
        self.key.clone()
    }
    fn render(&self) -> AnyView {
        let style = {
            let key = self.key.clone();
            let active_key = self.state.active_key.clone();
            move || {
                let is_active = active_key.read().as_ref() == Some(&key);
                let addon_style = if is_active {
                    "background-color: #EEE;"
                } else {
                    ""
                };
                format!(
                    "cursor: default;padding-top: 0.5em;padding-bottom: 0.5em;padding-left: 0.5em;{}",
                    addon_style
                )
            }
        };
        let name = self.name.clone();
        view! {
            <div style={style}>{name}</div>
        }
        .into_any()
    }
    fn children(&self) -> Option<Arc<Vec<MenuNode>>> {
        self.children.clone()
    }
    fn children_style(&self) -> Option<SharedString> {
        let base_style = "margin: 0;padding: 0;list-style-type: none;";
        let addon_style = if &Some(self.key.clone()) == self.state.expanded_key.read().deref() {
            "height: auto;"
        } else {
            "height: 0;overflow: hidden;"
        };
        Some(format!("{}{}", base_style, addon_style).into())
    }
}

#[component]
pub fn Menu(#[prop(into)] list: Signal<Vec<MenuNode>>) -> impl IntoView {
    let ul_style = "margin: 0;padding: 0;list-style-type: none;";
    view! {
        <ul style={ul_style}>
            <For
                each=move || { list.get().into_iter() }
                key=|node| { node.key.clone() }
                children=move |node| {
                    let onclick = UnsyncCallback::new(move |data: MenuNode| {
                        if data.children.is_none() {
                            if let Some(action) = data.action.as_ref() {
                                action.run(data.key.clone());
                            }
                        } else {
                            let mut expanded_key = data.state.expanded_key.write();
                            if Some(&data.key) == expanded_key.as_ref() {
                                expanded_key.take();
                            } else {
                                expanded_key.replace(data.key.clone());
                            }
                        }
                    });
                    view! {
                        <TreeNode<MenuNode>
                            data={node.clone()}
                            onclick={Some(onclick)}
                        />
                    }
                }
            />
        </ul>
    }
}
