use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub config_map_name: String,
    pub key: String,
    pub json_path: String,
    pub json_value: bool,
    pub value: String,
}

impl Parameter {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let mut parameter = serde_json::from_value::<Self>(value).map_err(|_err| -> String {
            return String::from("参数格式不正确");
        })?;
        let config_map_name = parameter.config_map_name.trim().to_string();
        let key = parameter.key.trim().to_string();
        let json_path = parameter.json_path.trim().to_string();
        if config_map_name.is_empty() {
            return Err(String::from("ConfigMap名称不能为空"));
        }
        if key.is_empty() {
            return Err(String::from("key不能为空"));
        }
        if json_path.is_empty() {
            return Err(String::from("json path不能为空"));
        }
        parameter.config_map_name = config_map_name;
        parameter.key = key;
        parameter.json_path = json_path;
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("config_map_name"),
            name: String::from("ConfigMap名称"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("key"),
            name: String::from("key"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("json_path"),
            name: String::from("json path"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("json_value"),
            name: String::from("json值"),
            description: Some(String::from(
                "如果勾选，则按json格式解析，否则按字符串处理。",
            )),
            r#type: AttributeType::Bool,
            required: true,
        },
        Attribute {
            id: String::from("value"),
            name: String::from("值"),
            description: None,
            r#type: AttributeType::Code {
                language: String::from("javascript"),
            },
            required: false,
        },
    ];
}
