use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
}

pub fn configuration_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("access_key"),
            name: String::from("access key"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("secret_key"),
            name: String::from("secret key"),
            description: None,
            r#type: AttributeType::Password,
            required: true,
        },
        Attribute {
            id: String::from("endpoint"),
            name: String::from("endpoint"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("region"),
            name: String::from("region"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("bucket"),
            name: String::from("bucket"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
    ];
}
