use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub key: String,
    pub value: String,
}

impl Parameter {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let mut parameter = serde_json::from_value::<Self>(value).map_err(|_err| -> String {
            return String::from("参数格式不正确");
        })?;
        let key = parameter.key.trim().to_string();
        let value = parameter.value.trim().to_string();
        if key.is_empty() {
            return Err(String::from("key不能为空"));
        }
        if value.is_empty() {
            return Err(String::from("value不能为空"));
        }
        parameter.key = key;
        parameter.value = value;
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("key"),
            name: String::from("key"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("value"),
            name: String::from("value"),
            description: None,
            r#type: AttributeType::LongString,
            required: true,
        },
    ];
}
