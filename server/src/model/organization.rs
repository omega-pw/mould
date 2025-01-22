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
 * 组织列
 */
pub enum OrganizationProperty {
    Id(Id),
    Name(String),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for OrganizationProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			OrganizationProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			OrganizationProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			OrganizationProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			OrganizationProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 组织
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Organization {
    pub id: Id, //id
    pub name: String, //组织名称
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl Organization {
    pub fn into_properties(self) -> Vec<OrganizationProperty> {
        return vec![
			OrganizationProperty::Id(self.id),
			OrganizationProperty::Name(self.name),
			OrganizationProperty::CreatedTime(self.created_time),
			OrganizationProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<OrganizationProperty> for Organization {
    fn eq(&self, property: &OrganizationProperty) -> bool {
        match property {
			OrganizationProperty::Id(id) => id == &self.id,
			OrganizationProperty::Name(name) => name == &self.name,
			OrganizationProperty::CreatedTime(created_time) => created_time == &self.created_time,
			OrganizationProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct OrganizationOpt {
    pub id: Option<Id>,
    pub name: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl OrganizationOpt {
    pub fn empty() -> OrganizationOpt {
        return OrganizationOpt {
            id: None,
            name: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}