use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    pub script: String,
    pub shell: String,
    pub sudo: bool,
}

impl Parameter {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let mut parameter = serde_json::from_value::<Self>(value).map_err(|_err| -> String {
            return String::from("参数格式不正确");
        })?;
        let script = parameter.script.trim().to_string();
        if script.is_empty() {
            return Err(String::from("脚本不能为空"));
        }
        parameter.script = script;
        let mut shell = parameter.shell.trim();
        if shell.is_empty() {
            shell = "/bin/sh";
        }
        parameter.shell = shell.to_string();
        return Ok(parameter);
    }
}

pub fn parameter_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("script"),
            name: String::from("shell脚本"),
            description: Some(String::from(
                "如果以#!/bin/开头，则认为是完整的脚本，否则会加上对应shell的解释器头。",
            )),
            r#type: AttributeType::Code {
                language: String::from("shell"),
            },
            required: true,
        },
        Attribute {
            id: String::from("shell"),
            name: String::from("shell解释器"),
            description: Some(String::from("默认:/bin/sh")),
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("sudo"),
            name: String::from("以sudo方式运行"),
            description: None,
            r#type: AttributeType::Bool,
            required: true,
        },
    ];
}
