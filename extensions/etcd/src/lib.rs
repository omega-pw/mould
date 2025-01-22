struct EtcdExtension;
use config::Config;
use mould_extension_sdk::async_trait::async_trait;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::Context;
use mould_extension_sdk::Extension;
use mould_extension_sdk::Operation;
use operation::modify_json;
use operation::modify_json_custom;
use operation::put;
use operation::test;
mod config;
mod operation;

const EXTENSION_ID: &str = "mould.etcd";
const EXTENSION_NAME: &str = "etcd配置中心";

#[async_trait]
impl Extension for EtcdExtension {
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
        _context: &Context,
    ) -> Result<(), String> {
        return operation::await_future(test::handle(configuration)).await?;
    }
    fn validate_operation_parameter(
        &self,
        operation_id: &str,
        operation_parameter: Value,
    ) -> Result<(), String> {
        if "put" == operation_id {
            return put::parameter::Parameter::try_form_value(operation_parameter).map(|_| ());
        } else if "modify_json" == operation_id {
            return modify_json::parameter::Parameter::try_form_value(operation_parameter)
                .map(|_| ());
        } else if "modify_json_custom" == operation_id {
            return modify_json_custom::parameter::Parameter::try_form_value(operation_parameter)
                .map(|_| ());
        } else {
            return Err(String::from("没有此操作"));
        }
    }
    fn operations(&self) -> Vec<Operation> {
        return vec![
            Operation {
                id: String::from("put"),
                name: String::from("修改配置"),
                parameter_schema: put::parameter::parameter_schema(),
            },
            Operation {
                id: String::from("modify_json"),
                name: String::from("修改json配置"),
                parameter_schema: modify_json::parameter::parameter_schema(),
            },
            Operation {
                id: String::from("modify_json_custom"),
                name: String::from("修改json配置(高级)"),
                parameter_schema: modify_json_custom::parameter::parameter_schema(),
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
        if "put" == operation_id {
            return operation::await_future(put::handle(
                configuration,
                operation_parameter,
                append_log.clone(),
            ))
            .await?;
        } else if "modify_json" == operation_id {
            return operation::await_future(modify_json::handle(
                configuration,
                operation_parameter,
                context.clone(),
                append_log.clone(),
            ))
            .await?;
        } else if "modify_json_custom" == operation_id {
            return operation::await_future(modify_json_custom::handle(
                configuration,
                operation_parameter,
                context.clone(),
                append_log.clone(),
                resource_index,
            ))
            .await?;
        } else {
            return Err(String::from("没有此操作"));
        }
    }
}

mould_extension_sdk::plugin_implementation!(Extension, EtcdExtension);
