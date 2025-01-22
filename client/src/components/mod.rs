pub mod alert;
pub mod button;
pub mod button_group;
pub mod center_middle;
pub mod checkbox;
pub mod checkbox_group;
pub mod common_popup;
pub mod confirm;
pub mod datetime_picker;
pub mod dialog;
pub mod drawer;
pub mod file_upload;
pub mod files_upload;
pub mod focus_area;
pub mod hidden_file;
pub mod r#if;
pub mod image;
pub mod input;
pub mod loading;
pub mod menu;
pub mod mock_select;
pub mod modal_dialog;
pub mod monaco_editor;
pub mod page;
pub mod pagination;
pub mod popup_message;
pub mod radio_group;
pub mod required;
pub mod rich_text;
pub mod running;
pub mod selection;
pub mod show;
pub mod table;
pub mod textarea;
pub mod tree;
pub mod uploading_file;
pub mod uploading_files;
pub mod validate_wrapper;
pub mod word_limit_wrapper;
use js_sys::Promise;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use yew::prelude::Html;

pub trait SelectOption {
    type Value;
    fn value(&self) -> Self::Value;
    fn label(&self) -> yew::Html;
}

impl<V, O> SelectOption for (V, O)
where
    V: Clone,
    O: std::fmt::Display,
{
    type Value = V;
    fn value(&self) -> V {
        self.0.clone()
    }
    fn label(&self) -> yew::Html {
        yew::Html::from(self.1.to_string())
    }
}

pub trait Renderer<T> {
    fn render(&self, data: &T) -> Html;
}

impl<T, F> Renderer<T> for F
where
    F: Fn(&T) -> Html,
{
    fn render(&self, data: &T) -> Html {
        self(data)
    }
}

pub struct ArcRenderer<T> {
    pub inner: Arc<dyn Renderer<T>>,
}

impl<T, F: Fn(&T) -> Html + 'static> From<F> for ArcRenderer<T> {
    fn from(inner: F) -> Self {
        ArcRenderer {
            inner: Arc::new(inner),
        }
    }
}

impl<T> Clone for ArcRenderer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for ArcRenderer<T> {
    type Target = dyn Renderer<T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<T> PartialEq for ArcRenderer<T> {
    fn eq(&self, other: &ArcRenderer<T>) -> bool {
        let (ArcRenderer { inner }, ArcRenderer { inner: rhs }) = (self, other);
        Arc::ptr_eq(inner, rhs)
    }
}

impl<T> fmt::Debug for ArcRenderer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArcRenderer<_>")
    }
}

#[derive(Clone, Debug)]
pub struct HashingFile {
    pub file: web_sys::File,
    pub sha512: Promise,
}

impl PartialEq for HashingFile {
    fn eq(&self, other: &HashingFile) -> bool {
        return self.file == other.file;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum File {
    Remote {
        key: String,
        name: String,
        size: f64,
        mime_type: String,
    },
    Local(HashingFile),
}
