use super::ArcRenderer;
use crate::ArcFn;
use crate::Key;
use crate::SharedString;
use leptos::prelude::*;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

pub trait RowRenderer<T> {
    fn render(&self, data: &T, index: usize) -> AnyView;
}

impl<T, F> RowRenderer<T> for F
where
    F: Fn(&T, usize) -> AnyView,
{
    fn render(&self, data: &T, index: usize) -> AnyView {
        self(data, index)
    }
}

pub struct ArcRowRenderer<T> {
    pub inner: Arc<dyn RowRenderer<T> + Send + Sync>,
}

impl<T, F: Fn(&T, usize) -> AnyView + Send + Sync + 'static> From<F> for ArcRowRenderer<T> {
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

#[derive(Clone)]
pub struct Column<T: Clone> {
    pub key: Key,
    pub head: ArcRenderer<()>,
    pub row: ArcRowRenderer<T>,
    pub head_style: Option<SharedString>,
    pub data_style: Option<ArcFn<usize, SharedString>>,
}

#[component]
pub fn Table<T>(
    #[prop(into)] list: Signal<Vec<(Key, T)>>,
    #[prop(into)] columns: Signal<Vec<Column<T>>>,
    #[prop(into, optional)] style: SharedString,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    view! {
        <table class="e-table" style={style}>
            <thead>
                <tr>
                    <For
                        each={
                            let columns = columns.clone();
                            move || { columns.get() }
                        }
                        key=|column| { column.key.clone() }
                        children=move |column| {
                            view! {
                                <th class="e-table-hcell" style={column.head_style.clone()}>{column.head.render(&())}</th>
                            }
                        }
                    />
                </tr>
            </thead>
            <tbody>
                <tr>
                    <For
                        each=move || { list.get().into_iter().enumerate() }
                        key=|(_index, (key, _row))| { key.clone() }
                        children=move |(index, (_key, row))| {
                            view! {
                                <tr class="e-table-row">
                                    <For
                                        each={
                                            let columns = columns.clone();
                                            move || { columns.get() }
                                        }
                                        key=|column| { column.key.clone() }
                                        children=move |column| {
                                            let style = column.data_style.as_ref().map(|data_style| data_style(index));
                                            view! {
                                                <td class="e-table-cell" style={style}>{column.row.render(&row, index)}</td>
                                            }
                                        }
                                    />
                                </tr>
                            }
                        }
                    />
                </tr>
            </tbody>
        </table>
    }
}
