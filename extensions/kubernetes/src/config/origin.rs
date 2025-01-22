use mould_extension_sdk::Attribute;
use mould_extension_sdk::AttributeType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub kubeconfig: String,
    pub context: Option<String>,
    pub cluster: Option<String>,
    pub user: Option<String>,
    pub namespace: Option<String>,
}

pub fn configuration_schema() -> Vec<Attribute> {
    return vec![
        Attribute {
            id: String::from("kubeconfig"),
            name: String::from("kube配置"),
            description: None,
            r#type: AttributeType::LongString,
            required: true,
        },
        Attribute {
            id: String::from("context"),
            name: String::from("上下文"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("cluster"),
            name: String::from("集群"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("user"),
            name: String::from("用户"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
        Attribute {
            id: String::from("namespace"),
            name: String::from("命名空间"),
            description: None,
            r#type: AttributeType::String,
            required: false,
        },
    ];
}
