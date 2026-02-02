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
pub mod table;
pub mod textarea;
pub mod tree;
pub mod uploading_file;
pub mod uploading_files;
pub mod validate_wrapper;
pub mod visable;
pub mod word_limit_wrapper;
use futures::channel::oneshot;
use js_sys::Promise;
use leptos::prelude::*;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use tihu::Handler;
use web_sys::File;

pub trait SelectOption {
    type Value;
    fn value(&self) -> Self::Value;
    fn label(&self) -> AnyView;
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
    fn label(&self) -> AnyView {
        self.1.to_string().into_any()
    }
}

pub trait Renderer<T: ?Sized> {
    fn render(&self, data: &T) -> AnyView;
}

impl<T, F> Renderer<T> for F
where
    T: ?Sized,
    F: Fn(&T) -> AnyView,
{
    fn render(&self, data: &T) -> AnyView {
        self(data)
    }
}

pub struct ArcRenderer<T: ?Sized> {
    pub inner: Arc<dyn Renderer<T> + Send + Sync>,
}

impl<T: ?Sized, F: Fn(&T) -> AnyView + Send + Sync + 'static> From<F> for ArcRenderer<T> {
    fn from(inner: F) -> Self {
        ArcRenderer {
            inner: Arc::new(inner),
        }
    }
}

impl<T: ?Sized> Clone for ArcRenderer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized> Deref for ArcRenderer<T> {
    type Target = dyn Renderer<T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<T: ?Sized> PartialEq for ArcRenderer<T> {
    fn eq(&self, other: &ArcRenderer<T>) -> bool {
        let (ArcRenderer { inner }, ArcRenderer { inner: rhs }) = (self, other);
        Arc::ptr_eq(inner, rhs)
    }
}

impl<T: ?Sized> fmt::Debug for ArcRenderer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArcRenderer<_>")
    }
}

#[derive(Clone)]
pub struct HashingFile {
    pub file: File,
    pub sha512: Promise,
}

impl PartialEq for HashingFile {
    fn eq(&self, other: &HashingFile) -> bool {
        return self.file == other.file;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ResourceMetadata {
    pub key: String,
    pub name: String,
    pub size: f64,
    pub mime_type: String,
}

#[derive(Clone, PartialEq)]
pub enum Resource {
    Remote(ResourceMetadata),
    Local(HashingFile),
}

pub struct ArcHandler<In, Out> {
    pub inner: Arc<dyn Handler<In, Out = Out>>,
}

impl<In, Out> Clone for ArcHandler<In, Out> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<In, Out> Deref for ArcHandler<In, Out> {
    type Target = dyn Handler<In, Out = Out>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<In, Out> PartialEq for ArcHandler<In, Out> {
    fn eq(&self, other: &ArcHandler<In, Out>) -> bool {
        let (ArcHandler { inner }, ArcHandler { inner: rhs }) = (self, other);
        Arc::ptr_eq(inner, rhs)
    }
}

impl<In, Out> fmt::Debug for ArcHandler<In, Out> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArcHandler<_,_>")
    }
}

pub fn on_cleanup_unsync(fun: impl FnOnce() + 'static) {
    let (sender, receiver) = oneshot::channel::<()>();
    wasm_bindgen_futures::spawn_local(async move {
        if receiver.await.is_ok() {
            fun();
        }
    });
    on_cleanup(move || {
        sender.send(()).ok();
    });
}

#[derive(Clone)]
struct LatestDestroy(RwSignal<Option<Box<dyn FnOnce()>>, LocalStorage>);

impl LatestDestroy {
    pub fn new() -> Self {
        Self(RwSignal::new_local(None))
    }
    pub fn replace(&self, destroy: impl FnOnce() + 'static) {
        let mut curr_destroy = self.0.write();
        if let Some(curr_destroy) = curr_destroy.take() {
            curr_destroy();
        }
        curr_destroy.replace(Box::new(destroy));
    }
    pub fn clear(&self) {
        let mut curr_destroy = self.0.write();
        if let Some(curr_destroy) = curr_destroy.take() {
            curr_destroy();
        }
    }
}

impl Default for LatestDestroy {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LatestDestroy {
    fn drop(&mut self) {
        self.clear();
    }
}
