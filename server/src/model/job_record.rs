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
    pub const JOB_ID: &str = "job_id";
    pub const ENVIRONMENT_ID: &str = "environment_id";
    pub const STATUS: &str = "status";
    pub const CREATED_TIME: &str = "created_time";
    pub const LAST_MODIFIED_TIME: &str = "last_modified_time";
}

pub mod enums {
    use tihu::LightString;
    use std::error::Error;
    use serde::{Serialize, Deserialize};
    use tokio_postgres::types::{ToSql, Type, IsNull, to_sql_checked};
    use bytes::BytesMut;
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Status {
        Running = 1, //进行中
        Success = 2, //成功
        Failure = 3, //失败
    }
    pub fn try_i16_to_status(val: i16) -> Result<Status, LightString> {
        match val {
            1 => Ok(Status::Running),
            2 => Ok(Status::Success),
            3 => Ok(Status::Failure),
            _ => Err(format!("未定义的状态枚举值:{}", val).into())
        }
    }
    impl ToSql for Status {
        fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + 'static + Send + Sync>> {
            (*self as i16).to_sql(ty, out)
        }
        fn accepts(ty: &Type) -> bool {
            <i16 as ToSql>::accepts(ty)
        }
        to_sql_checked!();
    }
}


/**
 * 任务记录列
 */
pub enum JobRecordProperty {
    Id(Id),
    OrgId(Id),
    JobId(Id),
    EnvironmentId(Id),
    Status(enums::Status),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for JobRecordProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			JobRecordProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobRecordProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobRecordProperty::JobId(_) => PropertyDefine {
                key: LightString::from_static(properties::JOB_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobRecordProperty::EnvironmentId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobRecordProperty::Status(_) => PropertyDefine {
                key: LightString::from_static(properties::STATUS),
                value_type: PropertyType::Enum,
				required: true,
            },
			JobRecordProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			JobRecordProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 任务记录
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct JobRecord {
    pub id: Id, //记录id
    pub org_id: Id, //组织id
    pub job_id: Id, //任务id
    pub environment_id: Id, //环境id
    pub status: enums::Status, //执行状态
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl JobRecord {
    pub fn into_properties(self) -> Vec<JobRecordProperty> {
        return vec![
			JobRecordProperty::Id(self.id),
			JobRecordProperty::OrgId(self.org_id),
			JobRecordProperty::JobId(self.job_id),
			JobRecordProperty::EnvironmentId(self.environment_id),
			JobRecordProperty::Status(self.status),
			JobRecordProperty::CreatedTime(self.created_time),
			JobRecordProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<JobRecordProperty> for JobRecord {
    fn eq(&self, property: &JobRecordProperty) -> bool {
        match property {
			JobRecordProperty::Id(id) => id == &self.id,
			JobRecordProperty::OrgId(org_id) => org_id == &self.org_id,
			JobRecordProperty::JobId(job_id) => job_id == &self.job_id,
			JobRecordProperty::EnvironmentId(environment_id) => environment_id == &self.environment_id,
			JobRecordProperty::Status(status) => status == &self.status,
			JobRecordProperty::CreatedTime(created_time) => created_time == &self.created_time,
			JobRecordProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct JobRecordOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub job_id: Option<Id>,
    pub environment_id: Option<Id>,
    pub status: Option<enums::Status>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl JobRecordOpt {
    pub fn empty() -> JobRecordOpt {
        return JobRecordOpt {
            id: None,
            org_id: None,
            job_id: None,
            environment_id: None,
            status: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}