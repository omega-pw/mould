pub mod query_extension;
pub mod test_configuration;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Attribute {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub r#type: AttributeType,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Operation {
    pub id: String,
    pub name: String,
    pub parameter_schema: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub configuration_schema: Vec<Attribute>,
    pub operations: Vec<Operation>,
}
