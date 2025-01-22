use crate::sdk;
use mould_extension_sdk;
use mould_extension_sdk::Extension;
use sdk::extension::Attribute;
use sdk::extension::AttributeType;
use sdk::extension::EnumOption;
use sdk::extension::Operation;

pub fn get_extension_info(extension: &dyn Extension) -> sdk::extension::Extension {
    let id = extension.id();
    let name = extension.name();
    let configuration_schema = extension.configuration_schema();
    let operations = extension.operations();
    return sdk::extension::Extension {
        id: id,
        name: name,
        configuration_schema: configuration_schema
            .into_iter()
            .map(to_sdk_config_field)
            .collect(),
        operations: operations
            .into_iter()
            .map(|operation| Operation {
                id: operation.id,
                name: operation.name,
                parameter_schema: operation
                    .parameter_schema
                    .into_iter()
                    .map(to_sdk_config_field)
                    .collect(),
            })
            .collect(),
    };
}

fn to_sdk_field_type(config_field: mould_extension_sdk::AttributeType) -> AttributeType {
    match config_field {
        mould_extension_sdk::AttributeType::String => AttributeType::String,
        mould_extension_sdk::AttributeType::StringList => AttributeType::StringList,
        mould_extension_sdk::AttributeType::LongString => AttributeType::LongString,
        // mould_extension_sdk::AttributeType::RichText => AttributeType::RichText,
        mould_extension_sdk::AttributeType::Code { language } => AttributeType::Code { language },
        mould_extension_sdk::AttributeType::Enum { options } => AttributeType::Enum {
            options: options
                .into_iter()
                .map(|option| EnumOption {
                    value: option.value,
                    label: option.label,
                })
                .collect(),
        },
        mould_extension_sdk::AttributeType::EnumList { options } => AttributeType::EnumList {
            options: options
                .into_iter()
                .map(|option| EnumOption {
                    value: option.value,
                    label: option.label,
                })
                .collect(),
        },
        mould_extension_sdk::AttributeType::Password => AttributeType::Password,
        mould_extension_sdk::AttributeType::Bool => AttributeType::Bool,
        mould_extension_sdk::AttributeType::File => AttributeType::File,
        mould_extension_sdk::AttributeType::FileList => AttributeType::FileList,
    }
}

fn to_sdk_config_field(config_field: mould_extension_sdk::Attribute) -> Attribute {
    Attribute {
        id: config_field.id,
        name: config_field.name,
        description: config_field.description,
        required: config_field.required,
        r#type: to_sdk_field_type(config_field.r#type),
    }
}
