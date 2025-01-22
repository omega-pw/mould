use super::AppendLog;
use super::Attribute;
use super::Context;
use super::Extension;
use super::Operation;
pub use async_trait;
use libloading::{Library, Symbol};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

#[macro_export]
macro_rules! plugin_trait {
    ($plugin_trait:path) => {
        pub unsafe fn load_plugin<Path: AsRef<std::path::Path>>(
            path: Path,
        ) -> Result<
            $crate::pluginator::LoadedPlugin<dyn $plugin_trait>,
            $crate::pluginator::LoadingError,
        > {
            unsafe { $crate::pluginator::load(path) }
        }
    };
}

#[macro_export]
macro_rules! plugin_implementation {
    ($plugin_trait:path, $initializer:expr) => {
        #[no_mangle]
        pub extern "C" fn get_interface(
        ) -> *mut $crate::pluginator::PluginWrapper<dyn $plugin_trait> {
            Box::into_raw(Box::new($crate::pluginator::PluginWrapper::new(Box::new(
                $crate::pluginator::ExtensionWrapper {
                    inner: ::std::sync::Arc::new($initializer),
                },
            ))))
        }
    };
}

pub struct PluginWrapper<Plugin: ?Sized> {
    inner: Box<Plugin>,
}

impl<Plugin: ?Sized> PluginWrapper<Plugin> {
    pub fn new(inner: Box<Plugin>) -> Self {
        Self { inner: inner }
    }
}

pub struct LoadedPlugin<Plugin: ?Sized> {
    library: ManuallyDrop<Library>,
    plugin: ManuallyDrop<Box<Plugin>>,
}

impl<Plugin: ?Sized> Drop for LoadedPlugin<Plugin> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.plugin);
            ManuallyDrop::drop(&mut self.library);
        }
    }
}

impl<Plugin: ?Sized> Deref for LoadedPlugin<Plugin> {
    type Target = Plugin;

    fn deref(&self) -> &Self::Target {
        self.plugin.as_ref()
    }
}

impl<Plugin: ?Sized> DerefMut for LoadedPlugin<Plugin> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.plugin.as_mut()
    }
}

#[derive(Debug)]
pub enum LoadingError {
    OpeningError(libloading::Error),
    InterfaceGettingError(libloading::Error),
}

pub unsafe fn load<Path: AsRef<std::path::Path>, Plugin: ?Sized>(
    path: Path,
) -> Result<LoadedPlugin<Plugin>, LoadingError> {
    let library =
        unsafe { Library::new(path.as_ref()) }.map_err(|e| LoadingError::OpeningError(e))?;
    let get_interface: Symbol<fn() -> *mut PluginWrapper<Plugin>> =
        unsafe { library.get(b"get_interface") }
            .map_err(|e| LoadingError::InterfaceGettingError(e))?;
    let plugin = unsafe { Box::from_raw(get_interface()) };
    Ok(LoadedPlugin {
        plugin: ManuallyDrop::new(plugin.inner),
        library: ManuallyDrop::new(library),
    })
}

pub struct ExtensionWrapper<T> {
    pub inner: Arc<T>,
}

#[async_trait::async_trait]
impl<T: Extension> Extension for ExtensionWrapper<T> {
    fn id(&self) -> String {
        self.inner.id()
    }
    fn name(&self) -> String {
        self.inner.name()
    }
    fn configuration_schema(&self) -> Vec<Attribute> {
        self.inner.configuration_schema()
    }
    fn validate_configuration(&self, configuration: Value) -> Result<(), String> {
        self.inner.validate_configuration(configuration)
    }
    async fn test_configuration(
        &self,
        configuration: Value,
        context: &Context,
    ) -> Result<(), String> {
        let _ = self
            .inner
            .test_configuration(configuration, context)
            .await?;
        Ok(())
    }
    fn validate_operation_parameter(
        &self,
        operation_id: &str,
        operation_parameter: Value,
    ) -> Result<(), String> {
        self.inner
            .validate_operation_parameter(operation_id, operation_parameter)
    }
    fn operations(&self) -> Vec<Operation> {
        self.inner.operations()
    }
    async fn handle(
        &self,
        configuration: Value,
        operation_id: &str,
        operation_parameter: Value,
        context: &Context,
        append_log: &AppendLog,
        resource_index: u32,
    ) -> Result<(), String> {
        self.inner
            .handle(
                configuration,
                operation_id,
                operation_parameter,
                context,
                append_log,
                resource_index,
            )
            .await
    }
}
