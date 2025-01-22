use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub data_id: String,
    pub group: String,
    pub content: String,
    pub content_type: String,
}

impl Parameter {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let mut parameter = serde_json::from_value::<Self>(value).map_err(|_err| -> String {
            return String::from("参数格式不正确");
        })?;
        let data_id = parameter.data_id.trim().to_string();
        let group = parameter.group.trim().to_string();
        let content = parameter.content.trim().to_string();
        let content_type = parameter.content_type.trim().to_string();
        if data_id.is_empty() {
            return Err(String::from("配置ID不能为空"));
        }
        if group.is_empty() {
            return Err(String::from("配置组不能为空"));
        }
        if content.is_empty() {
            return Err(String::from("内容不能为空"));
        }
        parameter.data_id = data_id;
        parameter.group = group;
        parameter.content = content;
        parameter.content_type = content_type;
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("data_id"),
            name: String::from("配置ID"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("group"),
            name: String::from("配置组"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("content"),
            name: String::from("内容"),
            description: None,
            r#type: AttributeType::LongString,
            required: true,
        },
        Attribute {
            id: String::from("content_type"),
            name: String::from("content type"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
    ];
}
