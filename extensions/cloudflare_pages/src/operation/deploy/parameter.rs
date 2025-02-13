use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use mould_extension_sdk::File;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub file: File,
}

impl Parameter {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let parameter = serde_json::from_value::<Self>(value).map_err(|_err| -> String {
            return String::from("参数格式不正确");
        })?;
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![Attribute {
        id: String::from("file"),
        name: String::from("文件"),
        description: None,
        r#type: AttributeType::File,
        required: true,
    }];
}
