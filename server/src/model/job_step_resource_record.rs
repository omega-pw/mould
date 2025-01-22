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
    pub const RECORD_ID: &str = "record_id";
    pub const JOB_STEP_RECORD_ID: &str = "job_step_record_id";
    pub const ENVIRONMENT_RESOURCE_ID: &str = "environment_resource_id";
    pub const RESOURCE_NAME: &str = "resource_name";
    pub const EXTENSION_CONFIGURATION: &str = "extension_configuration";
    pub const OUTPUT_FILE: &str = "output_file";
    pub const OUTPUT_CONTENT: &str = "output_content";
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
        Pending = 1, //未开始
        Running = 2, //进行中
        Success = 3, //成功
        Failure = 4, //失败
    }
    pub fn try_i16_to_status(val: i16) -> Result<Status, LightString> {
        match val {
            1 => Ok(Status::Pending),
            2 => Ok(Status::Running),
            3 => Ok(Status::Success),
            4 => Ok(Status::Failure),
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
 * 任务步骤资源记录列
 */
pub enum JobStepResourceRecordProperty {
    Id(Id),
    OrgId(Id),
    JobId(Id),
    EnvironmentId(Id),
    RecordId(Id),
    JobStepRecordId(Id),
    EnvironmentResourceId(Id),
    ResourceName(String),
    ExtensionConfiguration(String),
    OutputFile(Option<String>),
    OutputContent(Option<String>),
    Status(enums::Status),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for JobStepResourceRecordProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			JobStepResourceRecordProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepResourceRecordProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepResourceRecordProperty::JobId(_) => PropertyDefine {
                key: LightString::from_static(properties::JOB_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepResourceRecordProperty::EnvironmentId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepResourceRecordProperty::RecordId(_) => PropertyDefine {
                key: LightString::from_static(properties::RECORD_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepResourceRecordProperty::JobStepRecordId(_) => PropertyDefine {
                key: LightString::from_static(properties::JOB_STEP_RECORD_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepResourceRecordProperty::EnvironmentResourceId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_RESOURCE_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepResourceRecordProperty::ResourceName(_) => PropertyDefine {
                key: LightString::from_static(properties::RESOURCE_NAME),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepResourceRecordProperty::ExtensionConfiguration(_) => PropertyDefine {
                key: LightString::from_static(properties::EXTENSION_CONFIGURATION),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepResourceRecordProperty::OutputFile(_) => PropertyDefine {
                key: LightString::from_static(properties::OUTPUT_FILE),
                value_type: PropertyType::String,
				required: false,
            },
			JobStepResourceRecordProperty::OutputContent(_) => PropertyDefine {
                key: LightString::from_static(properties::OUTPUT_CONTENT),
                value_type: PropertyType::String,
				required: false,
            },
			JobStepResourceRecordProperty::Status(_) => PropertyDefine {
                key: LightString::from_static(properties::STATUS),
                value_type: PropertyType::Enum,
				required: true,
            },
			JobStepResourceRecordProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			JobStepResourceRecordProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 任务步骤资源记录
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct JobStepResourceRecord {
    pub id: Id, //步骤资源记录id
    pub org_id: Id, //组织id
    pub job_id: Id, //任务id
    pub environment_id: Id, //环境id
    pub record_id: Id, //记录id
    pub job_step_record_id: Id, //任务步骤记录id
    pub environment_resource_id: Id, //环境资源id
    pub resource_name: String, //资源名称
    pub extension_configuration: String, //扩展配置
    pub output_file: Option<String>, //日志文件
    pub output_content: Option<String>, //日志内容
    pub status: enums::Status, //执行状态
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl JobStepResourceRecord {
    pub fn into_properties(self) -> Vec<JobStepResourceRecordProperty> {
        return vec![
			JobStepResourceRecordProperty::Id(self.id),
			JobStepResourceRecordProperty::OrgId(self.org_id),
			JobStepResourceRecordProperty::JobId(self.job_id),
			JobStepResourceRecordProperty::EnvironmentId(self.environment_id),
			JobStepResourceRecordProperty::RecordId(self.record_id),
			JobStepResourceRecordProperty::JobStepRecordId(self.job_step_record_id),
			JobStepResourceRecordProperty::EnvironmentResourceId(self.environment_resource_id),
			JobStepResourceRecordProperty::ResourceName(self.resource_name),
			JobStepResourceRecordProperty::ExtensionConfiguration(self.extension_configuration),
			JobStepResourceRecordProperty::OutputFile(self.output_file),
			JobStepResourceRecordProperty::OutputContent(self.output_content),
			JobStepResourceRecordProperty::Status(self.status),
			JobStepResourceRecordProperty::CreatedTime(self.created_time),
			JobStepResourceRecordProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<JobStepResourceRecordProperty> for JobStepResourceRecord {
    fn eq(&self, property: &JobStepResourceRecordProperty) -> bool {
        match property {
			JobStepResourceRecordProperty::Id(id) => id == &self.id,
			JobStepResourceRecordProperty::OrgId(org_id) => org_id == &self.org_id,
			JobStepResourceRecordProperty::JobId(job_id) => job_id == &self.job_id,
			JobStepResourceRecordProperty::EnvironmentId(environment_id) => environment_id == &self.environment_id,
			JobStepResourceRecordProperty::RecordId(record_id) => record_id == &self.record_id,
			JobStepResourceRecordProperty::JobStepRecordId(job_step_record_id) => job_step_record_id == &self.job_step_record_id,
			JobStepResourceRecordProperty::EnvironmentResourceId(environment_resource_id) => environment_resource_id == &self.environment_resource_id,
			JobStepResourceRecordProperty::ResourceName(resource_name) => resource_name == &self.resource_name,
			JobStepResourceRecordProperty::ExtensionConfiguration(extension_configuration) => extension_configuration == &self.extension_configuration,
			JobStepResourceRecordProperty::OutputFile(output_file) => output_file == &self.output_file,
			JobStepResourceRecordProperty::OutputContent(output_content) => output_content == &self.output_content,
			JobStepResourceRecordProperty::Status(status) => status == &self.status,
			JobStepResourceRecordProperty::CreatedTime(created_time) => created_time == &self.created_time,
			JobStepResourceRecordProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct JobStepResourceRecordOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub job_id: Option<Id>,
    pub environment_id: Option<Id>,
    pub record_id: Option<Id>,
    pub job_step_record_id: Option<Id>,
    pub environment_resource_id: Option<Id>,
    pub resource_name: Option<String>,
    pub extension_configuration: Option<String>,
    pub output_file: Option<String>,
    pub output_content: Option<String>,
    pub status: Option<enums::Status>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl JobStepResourceRecordOpt {
    pub fn empty() -> JobStepResourceRecordOpt {
        return JobStepResourceRecordOpt {
            id: None,
            org_id: None,
            job_id: None,
            environment_id: None,
            record_id: None,
            job_step_record_id: None,
            environment_resource_id: None,
            resource_name: None,
            extension_configuration: None,
            output_file: None,
            output_content: None,
            status: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}