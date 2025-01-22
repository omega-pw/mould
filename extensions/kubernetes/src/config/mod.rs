mod origin;
use mould_extension_sdk::serde_json;
use mould_extension_sdk::serde_json::Value;
pub use origin::configuration_schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub kubeconfig: String,
    pub context: Option<String>,
    pub cluster: Option<String>,
    pub user: Option<String>,
    pub namespace: String,
}

impl Config {
    pub fn try_form_value(value: Value) -> Result<Self, String> {
        let origin_config =
            serde_json::from_value::<origin::Config>(value).map_err(|_err| -> String {
                return String::from("配置格式不正确");
            })?;
        let kubeconfig = origin_config.kubeconfig.trim();
        if kubeconfig.is_empty() {
            return Err(String::from("kube配置不能为空"));
        }
        let context = if let Some(context) = origin_config.context.as_ref() {
            let context = context.trim();
            if context.is_empty() {
                None
            } else {
                Some(context.to_string())
            }
        } else {
            None
        };
        let cluster = if let Some(cluster) = origin_config.cluster.as_ref() {
            let cluster = cluster.trim();
            if cluster.is_empty() {
                None
            } else {
                Some(cluster.to_string())
            }
        } else {
            None
        };
        let user = if let Some(user) = origin_config.user.as_ref() {
            let user = user.trim();
            if user.is_empty() {
                None
            } else {
                Some(user.to_string())
            }
        } else {
            None
        };
        let namespace = if let Some(namespace) = origin_config.namespace.as_ref() {
            let namespace = namespace.trim();
            if namespace.is_empty() {
                String::from("default")
            } else {
                namespace.to_string()
            }
        } else {
            String::from("default")
        };
        return Ok(Config {
            kubeconfig: kubeconfig.to_string(),
            context: context,
            cluster: cluster,
            user: user,
            namespace: namespace,
        });
    }
}
