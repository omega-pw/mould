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
 * 环境规格列
 */
pub enum EnvironmentSchemaProperty {
    Id(Id),
    OrgId(Id),
    Name(String),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for EnvironmentSchemaProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			EnvironmentSchemaProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentSchemaProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			EnvironmentSchemaProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			EnvironmentSchemaProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			EnvironmentSchemaProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 环境规格
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentSchema {
    pub id: Id, //id
    pub org_id: Id, //组织id
    pub name: String, //环境规格名称
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl EnvironmentSchema {
    pub fn into_properties(self) -> Vec<EnvironmentSchemaProperty> {
        return vec![
			EnvironmentSchemaProperty::Id(self.id),
			EnvironmentSchemaProperty::OrgId(self.org_id),
			EnvironmentSchemaProperty::Name(self.name),
			EnvironmentSchemaProperty::CreatedTime(self.created_time),
			EnvironmentSchemaProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<EnvironmentSchemaProperty> for EnvironmentSchema {
    fn eq(&self, property: &EnvironmentSchemaProperty) -> bool {
        match property {
			EnvironmentSchemaProperty::Id(id) => id == &self.id,
			EnvironmentSchemaProperty::OrgId(org_id) => org_id == &self.org_id,
			EnvironmentSchemaProperty::Name(name) => name == &self.name,
			EnvironmentSchemaProperty::CreatedTime(created_time) => created_time == &self.created_time,
			EnvironmentSchemaProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct EnvironmentSchemaOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub name: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl EnvironmentSchemaOpt {
    pub fn empty() -> EnvironmentSchemaOpt {
        return EnvironmentSchemaOpt {
            id: None,
            org_id: None,
            name: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}