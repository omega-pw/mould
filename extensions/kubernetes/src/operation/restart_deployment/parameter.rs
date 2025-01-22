use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub deployment_name: String,
}

impl Parameter {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let mut parameter = serde_json::from_value::<Self>(value).map_err(|_err| -> String {
            return String::from("参数格式不正确");
        })?;
        let deployment_name = parameter.deployment_name.trim().to_string();
        if deployment_name.is_empty() {
            return Err(String::from("工作负载名称不能为空"));
        }
        parameter.deployment_name = deployment_name;
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![Attribute {
        id: String::from("deployment_name"),
        name: String::from("工作负载名称"),
        description: None,
        r#type: AttributeType::String,
        required: true,
    }];
}
