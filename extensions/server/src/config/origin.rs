use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
    pub workspace: String,
}

pub fn configuration_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("host"),
            name: String::from("主机"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("port"),
            name: String::from("端口"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("user"),
            name: String::from("用户名"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("password"),
            name: String::from("密码"),
            description: None,
            r#type: AttributeType::Password,
            required: false,
        },
        Attribute {
            id: String::from("public_key"),
            name: String::from("公钥"),
            description: None,
            r#type: AttributeType::LongString,
            required: false,
        },
        Attribute {
            id: String::from("private_key"),
            name: String::from("私钥"),
            description: None,
            r#type: AttributeType::LongString,
            required: false,
        },
        Attribute {
            id: String::from("passphrase"),
            name: String::from("私钥密码"),
            description: None,
            r#type: AttributeType::Password,
            required: false,
        },
        Attribute {
            id: String::from("workspace"),
            name: String::from("工作目录"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
    ];
}
