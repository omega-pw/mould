pub use async_trait;
pub mod pluginator;
use serde::{Deserialize, Serialize};
pub use serde_json;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

pub struct Operation {
    pub id: String,
    pub name: String,
    pub parameter_schema: Vec<Attribute>,
}

pub struct EnumOption {
    pub value: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub key: String,
    pub name: String,
    pub size: f64,
    pub mime_type: String,
}

pub enum AttributeType {
    String,
    StringList,
    LongString,
    // RichText,
    Code { language: String },
    Password,
    Enum { options: Vec<EnumOption> },
    EnumList { options: Vec<EnumOption> },
    Bool,
    //序列化成对象，属性有key, name, size, mime_type
    File,
    //序列化成数组，数组对象的属性有key, name, size, mime_type
    FileList,
}

pub struct Attribute {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub r#type: AttributeType,
}

#[async_trait::async_trait]
pub trait ContextTrait {
    async fn spawn_blocking(&self, task: Box<dyn FnOnce() + Send + 'static>) -> Result<(), String>;
    fn spawn_future(
        &self,
        future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    ) -> Result<(), String>;
    fn modify_json_custom(
        &self,
        target: Value,
        json_path: &str,
        js_func: &str,
        resource_index: u32,
    ) -> Result<Value, String>;
    fn modify_json(
        &self,
        target: Value,
        json_path: &str,
        new_value: Value,
    ) -> Result<Value, String>;
    async fn download_file(&self, key: &str) -> Result<std::fs::File, String>;
}

pub type Context = Arc<dyn ContextTrait + Send + Sync>;
pub type AppendLog = Arc<dyn Fn(LogLevel, String) + Send + Sync>;

#[async_trait::async_trait]
pub trait Extension: Sync + Send + 'static {
    //扩展的id，类似包名，唯一表示该扩展
    fn id(&self) -> String;
    //扩展的名称
    fn name(&self) -> String;
    //扩展所需的配置
    fn configuration_schema(&self) -> Vec<Attribute>;
    //检查配置是否合格
    fn validate_configuration(&self, configuration: Value) -> Result<(), String>;
    //测试配置（主要是网络连通性）
    async fn test_configuration(
        &self,
        _configuration: Value,
        _context: &Context,
    ) -> Result<(), String> {
        return Ok(());
    }
    //检查操作参数是否合格
    fn validate_operation_parameter(
        &self,
        operation_id: &str,
        operation_parameter: Value,
    ) -> Result<(), String>;
    //该扩展可以执行哪些操作
    fn operations(&self) -> Vec<Operation>;
    //执行对应的操作
    async fn handle(
        &self,
        configuration: Value,
        operation_id: &str,
        operation_parameter: Value,
        context: &Context,
        append_log: &AppendLog,
        resource_index: u32,
    ) -> Result<(), String>;
}

plugin_trait!(Extension);
