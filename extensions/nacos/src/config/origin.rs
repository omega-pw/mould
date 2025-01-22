use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub server_addr: String,
    pub namespace: String,
    pub app_name: String,
    pub username: String,
    pub password: String,
}

pub fn configuration_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("server_addr"),
            name: String::from("服务器地址"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("namespace"),
            name: String::from("命名空间"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("app_name"),
            name: String::from("归属应用"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("username"),
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
