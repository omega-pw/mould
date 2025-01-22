use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub deployment_name: String,
    pub container_name: Option<String>,
    pub image: String,
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
        let container_name = parameter
            .container_name
            .map(|container_name| {
                let container_name = container_name.trim().to_string();
                if container_name.is_empty() {
                    None
                } else {
                    Some(container_name)
                }
            })
            .flatten();
        let image = parameter.image.trim().to_string();
        if image.is_empty() {
            return Err(String::from("镜像不能为空"));
        }
        parameter.deployment_name = deployment_name;
        parameter.container_name = container_name;
        parameter.image = image;
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("deployment_name"),
            name: String::from("工作负载名称"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("container_name"),
            name: String::from("容器名称"),
            description: Some(String::from("如果为空，则取第一个容器。")),
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("image"),
            name: String::from("镜像"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
    ];
}
