use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub endpoints: String,
    pub user: String,
    pub password: String,
}

pub fn configuration_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("endpoints"),
            name: String::from("endpoints"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("user"),
            name: String::from("用户名"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("password"),
            name: String::from("密码"),
            description: None,
            r#type: AttributeType::Password,
            required: false,
        },
    ];
}
