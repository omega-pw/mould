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
 * 环境列
 */
pub enum EnvironmentProperty {
    Id(Id),
    OrgId(Id),
    EnvironmentSchemaId(Id),
    Name(String),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for EnvironmentProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			EnvironmentProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentProperty::EnvironmentSchemaId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_SCHEMA_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			EnvironmentProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 环境
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
    pub id: Id, //id
    pub org_id: Id, //组织id
    pub environment_schema_id: Id, //环境规格id
    pub name: String, //环境名称
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl Environment {
    pub fn into_properties(self) -> Vec<EnvironmentProperty> {
        return vec![
			EnvironmentProperty::Id(self.id),
			EnvironmentProperty::OrgId(self.org_id),
			EnvironmentProperty::EnvironmentSchemaId(self.environment_schema_id),
			EnvironmentProperty::Name(self.name),
			EnvironmentProperty::CreatedTime(self.created_time),
			EnvironmentProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<EnvironmentProperty> for Environment {
    fn eq(&self, property: &EnvironmentProperty) -> bool {
        match property {
			EnvironmentProperty::Id(id) => id == &self.id,
			EnvironmentProperty::OrgId(org_id) => org_id == &self.org_id,
			EnvironmentProperty::EnvironmentSchemaId(environment_schema_id) => environment_schema_id == &self.environment_schema_id,
			EnvironmentProperty::Name(name) => name == &self.name,
			EnvironmentProperty::CreatedTime(created_time) => created_time == &self.created_time,
			EnvironmentProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct EnvironmentOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub environment_schema_id: Option<Id>,
    pub name: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl EnvironmentOpt {
    pub fn empty() -> EnvironmentOpt {
        return EnvironmentOpt {
            id: None,
            org_id: None,
            environment_schema_id: None,
            name: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}