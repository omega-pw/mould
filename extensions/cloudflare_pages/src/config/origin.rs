use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub api_token: String,
    pub account_id: String,
    pub project_name: String,
}

pub fn configuration_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("api_token"),
            name: String::from("Api Token"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("account_id"),
            name: String::from("Account Id"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
        Attribute {
            id: String::from("project_name"),
            name: String::from("Project Name"),
            description: None,
            r#type: AttributeType::String,
            required: true,
        },
    ];
}
