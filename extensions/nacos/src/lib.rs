struct NacosExtension;
use config::Config;
use mould_extension_sdk::async_trait::async_trait;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::Context;
use mould_extension_sdk::Extension;
use mould_extension_sdk::Operation;
use operation::modify_properties;
use operation::put;
use operation::test;
mod config;
mod operation;

const EXTENSION_ID: &str = "mould.nacos";
const EXTENSION_NAME: &str = "nacos配置中心";

#[async_trait]
impl Extension for NacosExtension {
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
        return test::handle(configuration).await;
    }
    fn validate_operation_parameter(
        &self,
        operation_id: &str,
        operation_parameter: Value,
    ) -> Result<(), String> {
        if "put" == operation_id {
            return put::parameter::Parameter::try_form_value(operation_parameter).map(|_| ());
        } else if "modify_properties" == operation_id {
            return modify_properties::parameter::Parameter::try_form_value(operation_parameter)
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
                id: String::from("modify_properties"),
                name: String::from("修改properties配置"),
                parameter_schema: modify_properties::parameter::parameter_schema(),
            },
        ];
    }
    async fn handle(
        &self,
        configuration: Value,
        operation_id: &str,
        operation_parameter: Value,
        _context: &Context,
        append_log: &AppendLog,
        _resource_index: u32,
    ) -> Result<(), String> {
        if "put" == operation_id {
            return put::handle(configuration, operation_parameter, append_log).await;
        } else if "modify_properties" == operation_id {
            return modify_properties::handle(configuration, operation_parameter, append_log).await;
        } else {
            return Err(String::from("没有此操作"));
        }
    }
}

mould_extension_sdk::plugin_implementation!(Extension, NacosExtension);
