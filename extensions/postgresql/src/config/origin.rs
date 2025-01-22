use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use mould_extension_sdk::File;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub dbname: String,
    pub user: String,
    pub password: String,
    pub ssl: bool,
    pub root_cert: Option<File>,
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
            id: String::from("dbname"),
            name: String::from("数据库"),
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
            required: true,
        },
        Attribute {
            id: String::from("ssl"),
            name: String::from("ssl"),
            description: None,
            r#type: AttributeType::Bool,
            required: true,
        },
        Attribute {
            id: String::from("root_cert"),
            name: String::from("根证书"),
            description: None,
            r#type: AttributeType::File,
            required: false,
        },
    ];
}
