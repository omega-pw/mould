use super::r#if::If;
use yew::prelude::*;
use yew::virtual_dom::Key;

pub trait Node {
    fn key(&self) -> Key;
    fn render(&self) -> Html;
    fn children(&self) -> Option<&[Self]>
    where
        Self: Sized;
    fn active(&self) -> bool {
        true
    }
    fn children_style(&self) -> Option<AttrValue> {
        None
    }
    fn children_class(&self) -> Option<AttrValue> {
        None
    }
    fn style(&self) -> Option<AttrValue> {
        None
    }
    fn class(&self) -> Option<AttrValue> {
        None
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T: Node + Clone + PartialEq> {
    pub data: T,
    #[prop_or_default]
    pub onclick: Option<Callback<T>>,
}

#[function_component]
pub fn TreeNode<T: Node + Clone + PartialEq + 'static>(props: &Props<T>) -> Html {
    html! {
        <If condition={props.data.active()}>
            {
                if let Some(children) = props.data.children() {
                    //有子节点
                    let has_active_children = children.iter().any(|child| child.active());
                    if has_active_children {
                        let onclick = props.onclick.clone();
                        let data = props.data.clone();
                        let on_click = Callback::from(move |_| {
                            if let Some(onclick) = onclick.as_ref() {
                                onclick.emit(data.clone());
                            }
                        });
                        html! {
                            <li key={props.data.key()} style={props.data.style()} class={props.data.class()}>
                                <div onclick={on_click}>{ props.data.render() }</div>
                                <ul style={props.data.children_style()} class={props.data.children_class()}>
                                    {
                                        for children.iter().map(|data| {
                                            html! {
                                                <TreeNode<T>
                                                    key={data.key()}
                                                    data={data.clone()}
                                                    onclick={props.onclick.clone()}
                                                />
                                            }
                                        })
                                    }
                                </ul>
                            </li>
                        }
                    } else {
                        html! {}
                    }
                } else {
                    //叶子结点
                    let onclick = props.onclick.clone();
                    let data = props.data.clone();
                    let on_click = Callback::from(move |_| {
                        if let Some(onclick) = onclick.as_ref() {
                            onclick.emit(data.clone());
                        }
                    });
                    html! {
                        <li key={props.data.key()} style={props.data.style()} class={props.data.class()}>
                            <div onclick={on_click}>{ props.data.render() }</div>
                        </li>
                    }
                }
            }
        </If>
    }
}
