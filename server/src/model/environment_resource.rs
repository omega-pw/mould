use serde::{Serialize, Deserialize};
use chrono::DateTime;
use chrono::Utc;
use tihu::datetime_format;
use tihu::datetime_format_opt;
use tihu::Id;
use tihu::LightString;
use native_common::model::Property;
use native_common::model::PropertyDefine;
use native_common::model::PropertyType;
use crate::native_common;

pub mod properties {
    pub const ID: &str = "id";
    pub const ORG_ID: &str = "org_id";
    pub const ENVIRONMENT_ID: &str = "environment_id";
    pub const SCHEMA_RESOURCE_ID: &str = "schema_resource_id";
    pub const NAME: &str = "name";
    pub const EXTENSION_ID: &str = "extension_id";
    pub const EXTENSION_NAME: &str = "extension_name";
    pub const EXTENSION_CONFIGURATION: &str = "extension_configuration";
    pub const CREATED_TIME: &str = "created_time";
    pub const LAST_MODIFIED_TIME: &str = "last_modified_time";
}

pub mod enums {
    use tihu::LightString;
    use std::error::Error;
    use serde::{Serialize, Deserialize};
    use tokio_postgres::types::{ToSql, Type, IsNull, to_sql_checked};
    use bytes::BytesMut;
}


/**
 * 环境资源列
 */
pub enum EnvironmentResourceProperty {
    Id(Id),
    OrgId(Id),
    EnvironmentId(Id),
    SchemaResourceId(Id),
    Name(String),
    ExtensionId(String),
    ExtensionName(String),
    ExtensionConfiguration(String),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for EnvironmentResourceProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			EnvironmentResourceProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentResourceProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentResourceProperty::EnvironmentId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentResourceProperty::SchemaResourceId(_) => PropertyDefine {
                key: LightString::from_static(properties::SCHEMA_RESOURCE_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentResourceProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentResourceProperty::ExtensionId(_) => PropertyDefine {
                key: LightString::from_static(properties::EXTENSION_ID),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentResourceProperty::ExtensionName(_) => PropertyDefine {
                key: LightString::from_static(properties::EXTENSION_NAME),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentResourceProperty::ExtensionConfiguration(_) => PropertyDefine {
                key: LightString::from_static(properties::EXTENSION_CONFIGURATION),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentResourceProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			EnvironmentResourceProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 环境资源
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentResource {
    pub id: Id, //id
    pub org_id: Id, //组织id
    pub environment_id: Id, //环境id
    pub schema_resource_id: Id, //环境规格资源id
    pub name: String, //资源名称
    pub extension_id: String, //扩展id
    pub extension_name: String, //扩展名称
    pub extension_configuration: String, //扩展配置
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl EnvironmentResource {
    pub fn into_properties(self) -> Vec<EnvironmentResourceProperty> {
        return vec![
			EnvironmentResourceProperty::Id(self.id),
			EnvironmentResourceProperty::OrgId(self.org_id),
			EnvironmentResourceProperty::EnvironmentId(self.environment_id),
			EnvironmentResourceProperty::SchemaResourceId(self.schema_resource_id),
			EnvironmentResourceProperty::Name(self.name),
			EnvironmentResourceProperty::ExtensionId(self.extension_id),
			EnvironmentResourceProperty::ExtensionName(self.extension_name),
			EnvironmentResourceProperty::ExtensionConfiguration(self.extension_configuration),
			EnvironmentResourceProperty::CreatedTime(self.created_time),
			EnvironmentResourceProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<EnvironmentResourceProperty> for EnvironmentResource {
    fn eq(&self, property: &EnvironmentResourceProperty) -> bool {
        match property {
			EnvironmentResourceProperty::Id(id) => id == &self.id,
			EnvironmentResourceProperty::OrgId(org_id) => org_id == &self.org_id,
			EnvironmentResourceProperty::EnvironmentId(environment_id) => environment_id == &self.environment_id,
			EnvironmentResourceProperty::SchemaResourceId(schema_resource_id) => schema_resource_id == &self.schema_resource_id,
			EnvironmentResourceProperty::Name(name) => name == &self.name,
			EnvironmentResourceProperty::ExtensionId(extension_id) => extension_id == &self.extension_id,
			EnvironmentResourceProperty::ExtensionName(extension_name) => extension_name == &self.extension_name,
			EnvironmentResourceProperty::ExtensionConfiguration(extension_configuration) => extension_configuration == &self.extension_configuration,
			EnvironmentResourceProperty::CreatedTime(created_time) => created_time == &self.created_time,
			EnvironmentResourceProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct EnvironmentResourceOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub environment_id: Option<Id>,
    pub schema_resource_id: Option<Id>,
    pub name: Option<String>,
    pub extension_id: Option<String>,
    pub extension_name: Option<String>,
    pub extension_configuration: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl EnvironmentResourceOpt {
    pub fn empty() -> EnvironmentResourceOpt {
        return EnvironmentResourceOpt {
            id: None,
            org_id: None,
            environment_id: None,
            schema_resource_id: None,
            name: None,
            extension_id: None,
            extension_name: None,
            extension_configuration: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}