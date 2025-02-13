struct CloudflarePagesExtension;
use config::Config;
use mould_extension_sdk::async_trait::async_trait;
use mould_extension_sdk::serde_json::Value;
use mould_extension_sdk::AppendLog;
use mould_extension_sdk::Attribute;
use mould_extension_sdk::Context;
use mould_extension_sdk::Extension;
use mould_extension_sdk::Operation;
use operation::deploy;
use operation::test;
mod config;
mod operation;

const EXTENSION_ID: &str = "mould.cloudflare.pages";
const EXTENSION_NAME: &str = "Cloudflare Pages";

#[async_trait]
impl Extension for CloudflarePagesExtension {
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
        if "deploy" == operation_id {
            return deploy::parameter::Parameter::try_form_value(operation_parameter).map(|_| ());
        }
        return Err(String::from("没有此操作"));
    }
    fn operations(&self) -> Vec<Operation> {
        return vec![Operation {
            id: String::from("deploy"),
            name: String::from("部署"),
            parameter_schema: deploy::parameter::parameter_schema(),
        }];
    }
    async fn handle(
        &self,
        configuration: Value,
        operation_id: &str,
        operation_parameter: Value,
        context: &Context,
        append_log: &AppendLog,
        _resource_index: u32,
    ) -> Result<(), String> {
        if "deploy" == operation_id {
            return deploy::handle(configuration, operation_parameter, context, append_log).await;
        }
        return Err(String::from("没有此操作"));
    }
}

mould_extension_sdk::plugin_implementation!(Extension, CloudflarePagesExtension);
