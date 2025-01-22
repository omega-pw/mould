use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub sql: String,
}

impl Parameter {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let mut parameter = serde_json::from_value::<Self>(value).map_err(|_err| -> String {
            return String::from("参数格式不正确");
        })?;
        let sql = parameter.sql.trim().to_string();
        if sql.is_empty() {
            return Err(String::from("sql不能为空"));
        }
        parameter.sql = sql;
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![Attribute {
        id: String::from("sql"),
        name: String::from("sql"),
        description: None,
        r#type: AttributeType::Code {
            language: String::from("sql"),
        },
        required: true,
    }];
}
