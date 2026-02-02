use crate::Key;
use crate::SharedString;
use leptos::prelude::*;
use std::sync::Arc;

pub trait Node {
    fn key(&self) -> Key;
    fn render(&self) -> AnyView;
    fn children(&self) -> Option<Arc<Vec<Self>>>
    where
        Self: Sized;
    fn active(&self) -> bool {
        true
    }
    fn children_style(&self) -> Option<SharedString> {
        None
    }
    fn children_class(&self) -> Option<SharedString> {
        None
    }
    fn style(&self) -> Option<SharedString> {
        None
    }
    fn class(&self) -> Option<SharedString> {
        None
    }
}

#[component]
pub fn TreeNode<T>(data: T, #[prop(default = None)] onclick: Option<UnsyncCallback<T>>) -> AnyView
where
    T: Node + Clone + Send + Sync + 'static,
{
    view! {
        <Show
            when= {
                let data = data.clone();
                move || { data.active() }
            }
        >
            {
                if let Some(children) = data.children() {
                    //有子节点
                    let has_active_children = children.iter().any(|child| child.active());
                    if has_active_children {
                        let on_click = {
                            let onclick = onclick.clone();
                            let data = data.clone();
                            move |_| {
                                if let Some(onclick) = onclick.as_ref() {
                                    onclick.run(data.clone());
                                }
                            }
                        };
                        view! {
                            <li style={data.style()} class={data.class()}>
                                <div on:click={on_click}>{ data.render() }</div>
                                <ul style={data.children_style()} class={data.children_class()}>
                                    <For
                                        each=move || { children.as_ref().clone().into_iter() }
                                        key=|child| { child.key() }
                                        children=move |child| {
                                            view! {
                                                <TreeNode<T>
                                                    data={child}
                                                    onclick={onclick.clone()}
                                                />
                                            }
                                        }
                                    />
                                </ul>
                            </li>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                } else {
                    //叶子结点
                    let on_click = {
                        let data = data.clone();
                        move |_| {
                            if let Some(onclick) = onclick.as_ref() {
                                onclick.run(data.clone());
                            }
                        }
                    };
                    view! {
                        <li style={data.style()} class={data.class()}>
                            <div on:click={on_click}>{ data.render() }</div>
                        </li>
                    }.into_any()
                }
            }
        </Show>
    }
    .into_any()
}
