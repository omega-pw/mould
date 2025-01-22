use super::binding::Binding;
use super::gen_id;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use yew::prelude::*;
use yew::virtual_dom::Key;

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T: Clone + PartialEq> {
    pub items: UseStateHandle<Vec<(Key, Binding<T>)>>,
    pub renderer: ArcRowRenderer<T>,
}

#[function_component]
pub fn List<T>(props: &Props<T>) -> Html
where
    T: Clone + PartialEq + 'static,
{
    let items_clone = props.items.clone();
    html! {
        for props.items.iter().enumerate().map(|(index, (key, value))| {
            let items = items_clone.clone();
            let on_remove = Callback::from(move |_| {
                let mut new_items = items.deref().clone();
                new_items.remove(index);
                items.set(new_items);
            });
            html! {
                <Row<T> key={key.clone()} value={value.clone()} index={index} onremove={on_remove} renderer={props.renderer.clone()} />
            }
        })
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct RowProps<T: Clone + PartialEq> {
    pub value: Binding<T>,
    pub index: usize,
    pub onremove: Callback<()>,
    pub renderer: ArcRowRenderer<T>,
}

#[function_component]
pub fn Row<T>(props: &RowProps<T>) -> Html
where
    T: Clone + PartialEq + 'static,
{
    let index = props.index;
    let onremove = props.onremove.clone();
    let renderer = props.renderer.clone();
    props
        .value
        .view(move |value| renderer.render(value, index, onremove.clone()))
}

pub trait RowRenderer<T> {
    fn render(&self, value: UseStateHandle<T>, index: usize, onremove: Callback<()>) -> Html;
}

impl<T, F> RowRenderer<T> for F
where
    F: Fn(UseStateHandle<T>, usize, Callback<()>) -> Html,
{
    fn render(&self, value: UseStateHandle<T>, index: usize, onremove: Callback<()>) -> Html {
        self(value, index, onremove)
    }
}

pub struct ArcRowRenderer<T> {
    pub inner: Arc<dyn RowRenderer<T>>,
}

impl<T, F: Fn(UseStateHandle<T>, usize, Callback<()>) -> Html + 'static> From<F>
    for ArcRowRenderer<T>
{
    fn from(inner: F) -> Self {
        ArcRowRenderer {
            inner: Arc::new(inner),
        }
    }
}

impl<T> Clone for ArcRowRenderer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for ArcRowRenderer<T> {
    type Target = dyn RowRenderer<T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<T> PartialEq for ArcRowRenderer<T> {
    fn eq(&self, other: &ArcRowRenderer<T>) -> bool {
        let (ArcRowRenderer { inner }, ArcRowRenderer { inner: rhs }) = (self, other);
        Arc::ptr_eq(inner, rhs)
    }
}

impl<T> fmt::Debug for ArcRowRenderer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArcRowRenderer<_>")
    }
}

pub fn list_view<T>(
    list: UseStateHandle<Vec<(Key, Binding<T>)>>,
    renderer: impl RowRenderer<T> + 'static,
) -> (Html, Callback<T>)
where
    T: Clone + PartialEq + 'static,
{
    let list_clone = list.clone();
    let on_add = Callback::from(move |item: T| {
        let mut new_list = list_clone.deref().clone();
        new_list.push((gen_id().into(), Binding::new(item)));
        list_clone.set(new_list);
    });
    let renderer = ArcRowRenderer {
        inner: Arc::new(renderer),
    };
    let view = html! {
        <List<T> items={list} renderer={renderer}/>
    };
    return (view, on_add);
}
