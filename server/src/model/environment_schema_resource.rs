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
    pub const ENVIRONMENT_SCHEMA_ID: &str = "environment_schema_id";
    pub const NAME: &str = "name";
    pub const EXTENSION_ID: &str = "extension_id";
    pub const EXTENSION_NAME: &str = "extension_name";
    pub const CREATED_TIME: &str = "created_time";
    pub const LAST_MODIFIED_TIME: &str = "last_modified_time";
}




/**
 * 环境规格资源列
 */
pub enum EnvironmentSchemaResourceProperty {
    Id(Id),
    OrgId(Id),
    EnvironmentSchemaId(Id),
    Name(String),
    ExtensionId(String),
    ExtensionName(String),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for EnvironmentSchemaResourceProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			EnvironmentSchemaResourceProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentSchemaResourceProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentSchemaResourceProperty::EnvironmentSchemaId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_SCHEMA_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentSchemaResourceProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentSchemaResourceProperty::ExtensionId(_) => PropertyDefine {
                key: LightString::from_static(properties::EXTENSION_ID),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentSchemaResourceProperty::ExtensionName(_) => PropertyDefine {
                key: LightString::from_static(properties::EXTENSION_NAME),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentSchemaResourceProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			EnvironmentSchemaResourceProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 环境规格资源
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentSchemaResource {
    pub id: Id, //id
    pub org_id: Id, //组织id
    pub environment_schema_id: Id, //环境规格id
    pub name: String, //资源名称
    pub extension_id: String, //扩展id
    pub extension_name: String, //扩展名称
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl EnvironmentSchemaResource {
    pub fn into_properties(self) -> Vec<EnvironmentSchemaResourceProperty> {
        return vec![
			EnvironmentSchemaResourceProperty::Id(self.id),
			EnvironmentSchemaResourceProperty::OrgId(self.org_id),
			EnvironmentSchemaResourceProperty::EnvironmentSchemaId(self.environment_schema_id),
			EnvironmentSchemaResourceProperty::Name(self.name),
			EnvironmentSchemaResourceProperty::ExtensionId(self.extension_id),
			EnvironmentSchemaResourceProperty::ExtensionName(self.extension_name),
			EnvironmentSchemaResourceProperty::CreatedTime(self.created_time),
			EnvironmentSchemaResourceProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<EnvironmentSchemaResourceProperty> for EnvironmentSchemaResource {
    fn eq(&self, property: &EnvironmentSchemaResourceProperty) -> bool {
        match property {
			EnvironmentSchemaResourceProperty::Id(id) => id == &self.id,
			EnvironmentSchemaResourceProperty::OrgId(org_id) => org_id == &self.org_id,
			EnvironmentSchemaResourceProperty::EnvironmentSchemaId(environment_schema_id) => environment_schema_id == &self.environment_schema_id,
			EnvironmentSchemaResourceProperty::Name(name) => name == &self.name,
			EnvironmentSchemaResourceProperty::ExtensionId(extension_id) => extension_id == &self.extension_id,
			EnvironmentSchemaResourceProperty::ExtensionName(extension_name) => extension_name == &self.extension_name,
			EnvironmentSchemaResourceProperty::CreatedTime(created_time) => created_time == &self.created_time,
			EnvironmentSchemaResourceProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct EnvironmentSchemaResourceOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub environment_schema_id: Option<Id>,
    pub name: Option<String>,
    pub extension_id: Option<String>,
    pub extension_name: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl EnvironmentSchemaResourceOpt {
    pub fn empty() -> EnvironmentSchemaResourceOpt {
        return EnvironmentSchemaResourceOpt {
            id: None,
            org_id: None,
            environment_schema_id: None,
            name: None,
            extension_id: None,
            extension_name: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}