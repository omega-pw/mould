use std::sync::Arc;
use std::sync::RwLock;

pub trait SafeHandle<T>: Clone {
    fn get<N>(&self, getter: impl FnOnce(&T) -> N) -> N;
    fn set(&self, value: T);
}

pub struct ArcHandle<T> {
    data: Arc<RwLock<T>>,
    listener: Option<Arc<dyn Fn(T)>>,
}

impl<T> ArcHandle<T> {
    pub fn new_with_listener(data: T, listener: impl Fn(T) + 'static) -> Self {
        ArcHandle {
            data: Arc::new(RwLock::new(data)),
            listener: Some(Arc::new(listener)),
        }
    }
    pub fn new(data: T) -> Self {
        ArcHandle {
            data: Arc::new(RwLock::new(data)),
            listener: None,
        }
    }
}

impl<T> Clone for ArcHandle<T> {
    fn clone(&self) -> Self {
        ArcHandle {
            data: self.data.clone(),
            listener: self.listener.clone(),
        }
    }
}

impl<T> SafeHandle<T> for ArcHandle<T>
where
    T: Clone,
{
    fn get<N>(&self, getter: impl FnOnce(&T) -> N) -> N {
        let data = self.data.read().unwrap();
        getter(&data)
    }
    fn set(&self, value: T) {
        *self.data.write().unwrap() = value.clone();
        if let Some(listener) = self.listener.as_ref() {
            listener(value);
        }
    }
}
