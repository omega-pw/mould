struct KubernetesExtension;
use config::Config;
use mould_extension_sdk::async_trait::async_trait;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::Context;
use mould_extension_sdk::Extension;
use mould_extension_sdk::Operation;
use operation::modify_config_map_json;
use operation::modify_config_map_json_custom;
use operation::put_config_map;
use operation::restart_deployment;
use operation::set_image;
use operation::test;
mod config;
mod operation;

const EXTENSION_ID: &str = "mould.kubernetes";
const EXTENSION_NAME: &str = "kubernetes";

#[async_trait]
impl Extension for KubernetesExtension {
    fn id(&self) -> String {
        return String::from(EXTENSION_ID);
    }
    fn name(&self) -> String {
        return String::from(EXTENSION_NAME);
    }
    fn configuration_schema(&self) -> Vec<Attribute> {
        return config::configuration_schema();
    }
    fn validate_configuration(&self, configuration: Value) -> Result<(), String> {
        return Config::try_form_value(configuration).map(|_| ());
    }
    async fn test_configuration(
        &self,
        configuration: Value,
        context: &Context,
    ) -> Result<(), String> {
        return test::handle(configuration, context).await;
    }
    fn validate_operation_parameter(
        &self,
        operation_id: &str,
        operation_parameter: Value,
    ) -> Result<(), String> {
        if "set_image" == operation_id {
            return set_image::parameter::Parameter::try_form_value(operation_parameter)
                .map(|_| ());
        } else if "put_config_map" == operation_id {
            return put_config_map::parameter::Parameter::try_form_value(operation_parameter)
                .map(|_| ());
        } else if "modify_config_map_json" == operation_id {
            return modify_config_map_json::parameter::Parameter::try_form_value(
                operation_parameter,
            )
            .map(|_| ());
        } else if "modify_config_map_json_custom" == operation_id {
            return modify_config_map_json_custom::parameter::Parameter::try_form_value(
                operation_parameter,
            )
            .map(|_| ());
        } else if "restart_deployment" == operation_id {
            return restart_deployment::parameter::Parameter::try_form_value(operation_parameter)
                .map(|_| ());
        }
        return Err(String::from("没有此操作"));
    }
    fn operations(&self) -> Vec<Operation> {
        return vec![
            Operation {
                id: String::from("set_image"),
                name: String::from("设置镜像"),
                parameter_schema: set_image::parameter::parameter_schema(),
            },
            Operation {
                id: String::from("put_config_map"),
                name: String::from("修改ConfigMap配置"),
                parameter_schema: put_config_map::parameter::parameter_schema(),
            },
            Operation {
                id: String::from("modify_config_map_json"),
                name: String::from("修改ConfigMap的json配置"),
                parameter_schema: modify_config_map_json::parameter::parameter_schema(),
            },
            Operation {
                id: String::from("modify_config_map_json_custom"),
                name: String::from("修改ConfigMap的json配置(高级)"),
                parameter_schema: modify_config_map_json_custom::parameter::parameter_schema(),
            },
            Operation {
                id: String::from("restart_deployment"),
                name: String::from("重启工作负载"),
                parameter_schema: restart_deployment::parameter::parameter_schema(),
            },
        ];
    }
    async fn handle(
        &self,
        configuration: Value,
        operation_id: &str,
        operation_parameter: Value,
        context: &Context,
        append_log: &AppendLog,
        resource_index: u32,
    ) -> Result<(), String> {
        if "set_image" == operation_id {
            return set_image::handle(configuration, operation_parameter, context, append_log)
                .await;
        } else if "put_config_map" == operation_id {
            return put_config_map::handle(configuration, operation_parameter, context, append_log)
                .await;
        } else if "modify_config_map_json" == operation_id {
            return modify_config_map_json::handle(
                configuration,
                operation_parameter,
                context,
                append_log,
            )
            .await;
        } else if "modify_config_map_json_custom" == operation_id {
            return modify_config_map_json_custom::handle(
                configuration,
                operation_parameter,
                context,
                append_log,
                resource_index,
            )
            .await;
        } else if "restart_deployment" == operation_id {
            return restart_deployment::handle(
                configuration,
                operation_parameter,
                context,
                append_log,
            )
            .await;
        }
        return Err(String::from("没有此操作"));
    }
}

mould_extension_sdk::plugin_implementation!(Extension, KubernetesExtension);
