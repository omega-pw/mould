use super::ArcRenderer;
use crate::ArcFn;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use yew::prelude::*;
use yew::virtual_dom::Key;

pub trait RowRenderer<T> {
    fn render(&self, data: &T, index: usize) -> Html;
}

impl<T, F> RowRenderer<T> for F
where
    F: Fn(&T, usize) -> Html,
{
    fn render(&self, data: &T, index: usize) -> Html {
        self(data, index)
    }
}

pub struct ArcRowRenderer<T> {
    pub inner: Arc<dyn RowRenderer<T>>,
}

impl<T, F: Fn(&T, usize) -> Html + 'static> From<F> for ArcRowRenderer<T> {
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

#[derive(Clone, PartialEq, Properties)]
pub struct Column<T: Clone + PartialEq> {
    pub key: Key,
    pub head: ArcRenderer<()>,
    pub row: ArcRowRenderer<T>,
    pub head_style: Option<AttrValue>,
    pub data_style: Option<ArcFn<usize, AttrValue>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T: Clone + PartialEq> {
    pub list: Vec<(Key, T)>,
    pub columns: Vec<Column<T>>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
}

#[function_component]
pub fn Table<T: Clone + PartialEq>(props: &Props<T>) -> Html {
    html! {
        <table class="e-table" style={props.style.clone()}>
            <thead>
                <tr>
                    {
                        for props.columns.iter().map(|column| {
                            html! {
                                <th key={column.key.clone()} class="e-table-hcell" style={column.head_style.clone()}>{column.head.render(&())}</th>
                            }
                        })
                    }
                </tr>
            </thead>
            <tbody>
                {
                    for props.list.iter().enumerate().map(|(index, (key, row))| {
                        html!{
                            <tr key={key.clone()} class="e-table-row">
                                {
                                    for props.columns.iter().map(|column| {
                                        let style = column.data_style.as_ref().map(|data_style| data_style(index));
                                        html!{
                                            <td key={column.key.clone()} class="e-table-cell" style={style}>{column.row.render(row, index)}</td>
                                        }
                                    })
                                }
                            </tr>
                        }
                    })
                }
            </tbody>
        </table>
    }
}
