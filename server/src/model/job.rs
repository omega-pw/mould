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
    pub const REMARK: &str = "remark";
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
 * 任务列
 */
pub enum JobProperty {
    Id(Id),
    OrgId(Id),
    EnvironmentSchemaId(Id),
    Name(String),
    Remark(Option<String>),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for JobProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			JobProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobProperty::EnvironmentSchemaId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_SCHEMA_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			JobProperty::Remark(_) => PropertyDefine {
                key: LightString::from_static(properties::REMARK),
                value_type: PropertyType::String,
				required: false,
            },
			JobProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			JobProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 任务
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub id: Id, //任务id
    pub org_id: Id, //组织id
    pub environment_schema_id: Id, //环境规格id
    pub name: String, //任务名称
    pub remark: Option<String>, //备注
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl Job {
    pub fn into_properties(self) -> Vec<JobProperty> {
        return vec![
			JobProperty::Id(self.id),
			JobProperty::OrgId(self.org_id),
			JobProperty::EnvironmentSchemaId(self.environment_schema_id),
			JobProperty::Name(self.name),
			JobProperty::Remark(self.remark),
			JobProperty::CreatedTime(self.created_time),
			JobProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<JobProperty> for Job {
    fn eq(&self, property: &JobProperty) -> bool {
        match property {
			JobProperty::Id(id) => id == &self.id,
			JobProperty::OrgId(org_id) => org_id == &self.org_id,
			JobProperty::EnvironmentSchemaId(environment_schema_id) => environment_schema_id == &self.environment_schema_id,
			JobProperty::Name(name) => name == &self.name,
			JobProperty::Remark(remark) => remark == &self.remark,
			JobProperty::CreatedTime(created_time) => created_time == &self.created_time,
			JobProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct JobOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub environment_schema_id: Option<Id>,
    pub name: Option<String>,
    pub remark: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl JobOpt {
    pub fn empty() -> JobOpt {
        return JobOpt {
            id: None,
            org_id: None,
            environment_schema_id: None,
            name: None,
            remark: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}