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
    pub const JOB_STEP_ID: &str = "job_step_id";
    pub const STEP_NAME: &str = "step_name";
    pub const STEP_TYPE: &str = "step_type";
    pub const STEP_REMARK: &str = "step_remark";
    pub const EXTENSION_ID: &str = "extension_id";
    pub const OPERATION_ID: &str = "operation_id";
    pub const OPERATION_NAME: &str = "operation_name";
    pub const OPERATION_PARAMETER: &str = "operation_parameter";
    pub const ATTACHMENTS: &str = "attachments";
    pub const JOB_STEP_SEQ: &str = "job_step_seq";
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
    pub enum StepType {
        Auto = 1, //自动
        Manual = 2, //手动
    }
    pub fn try_i16_to_step_type(val: i16) -> Result<StepType, LightString> {
        match val {
            1 => Ok(StepType::Auto),
            2 => Ok(StepType::Manual),
            _ => Err(format!("未定义的步骤类型枚举值:{}", val).into())
        }
    }
    impl ToSql for StepType {
        fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + 'static + Send + Sync>> {
            (*self as i16).to_sql(ty, out)
        }
        fn accepts(ty: &Type) -> bool {
            <i16 as ToSql>::accepts(ty)
        }
        to_sql_checked!();
    }
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
 * 任务步骤记录列
 */
pub enum JobStepRecordProperty {
    Id(Id),
    OrgId(Id),
    JobId(Id),
    EnvironmentId(Id),
    RecordId(Id),
    JobStepId(Id),
    StepName(String),
    StepType(enums::StepType),
    StepRemark(Option<String>),
    ExtensionId(String),
    OperationId(String),
    OperationName(String),
    OperationParameter(String),
    Attachments(Option<String>),
    JobStepSeq(i32),
    Status(enums::Status),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for JobStepRecordProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			JobStepRecordProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepRecordProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepRecordProperty::JobId(_) => PropertyDefine {
                key: LightString::from_static(properties::JOB_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepRecordProperty::EnvironmentId(_) => PropertyDefine {
                key: LightString::from_static(properties::ENVIRONMENT_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepRecordProperty::RecordId(_) => PropertyDefine {
                key: LightString::from_static(properties::RECORD_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepRecordProperty::JobStepId(_) => PropertyDefine {
                key: LightString::from_static(properties::JOB_STEP_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepRecordProperty::StepName(_) => PropertyDefine {
                key: LightString::from_static(properties::STEP_NAME),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepRecordProperty::StepType(_) => PropertyDefine {
                key: LightString::from_static(properties::STEP_TYPE),
                value_type: PropertyType::Enum,
				required: true,
            },
			JobStepRecordProperty::StepRemark(_) => PropertyDefine {
                key: LightString::from_static(properties::STEP_REMARK),
                value_type: PropertyType::String,
				required: false,
            },
			JobStepRecordProperty::ExtensionId(_) => PropertyDefine {
                key: LightString::from_static(properties::EXTENSION_ID),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepRecordProperty::OperationId(_) => PropertyDefine {
                key: LightString::from_static(properties::OPERATION_ID),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepRecordProperty::OperationName(_) => PropertyDefine {
                key: LightString::from_static(properties::OPERATION_NAME),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepRecordProperty::OperationParameter(_) => PropertyDefine {
                key: LightString::from_static(properties::OPERATION_PARAMETER),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepRecordProperty::Attachments(_) => PropertyDefine {
                key: LightString::from_static(properties::ATTACHMENTS),
                value_type: PropertyType::String,
				required: false,
            },
			JobStepRecordProperty::JobStepSeq(_) => PropertyDefine {
                key: LightString::from_static(properties::JOB_STEP_SEQ),
                value_type: PropertyType::Integer,
				required: true,
            },
			JobStepRecordProperty::Status(_) => PropertyDefine {
                key: LightString::from_static(properties::STATUS),
                value_type: PropertyType::Enum,
				required: true,
            },
			JobStepRecordProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			JobStepRecordProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 任务步骤记录
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct JobStepRecord {
    pub id: Id, //步骤记录id
    pub org_id: Id, //组织id
    pub job_id: Id, //任务id
    pub environment_id: Id, //环境id
    pub record_id: Id, //记录id
    pub job_step_id: Id, //任务步骤id
    pub step_name: String, //步骤名称
    pub step_type: enums::StepType, //步骤类型
    pub step_remark: Option<String>, //步骤备注
    pub extension_id: String, //扩展id
    pub operation_id: String, //操作id
    pub operation_name: String, //操作名称
    pub operation_parameter: String, //操作参数
    pub attachments: Option<String>, //附件
    pub job_step_seq: i32, //任务步骤顺序
    pub status: enums::Status, //执行状态
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl JobStepRecord {
    pub fn into_properties(self) -> Vec<JobStepRecordProperty> {
        return vec![
			JobStepRecordProperty::Id(self.id),
			JobStepRecordProperty::OrgId(self.org_id),
			JobStepRecordProperty::JobId(self.job_id),
			JobStepRecordProperty::EnvironmentId(self.environment_id),
			JobStepRecordProperty::RecordId(self.record_id),
			JobStepRecordProperty::JobStepId(self.job_step_id),
			JobStepRecordProperty::StepName(self.step_name),
			JobStepRecordProperty::StepType(self.step_type),
			JobStepRecordProperty::StepRemark(self.step_remark),
			JobStepRecordProperty::ExtensionId(self.extension_id),
			JobStepRecordProperty::OperationId(self.operation_id),
			JobStepRecordProperty::OperationName(self.operation_name),
			JobStepRecordProperty::OperationParameter(self.operation_parameter),
			JobStepRecordProperty::Attachments(self.attachments),
			JobStepRecordProperty::JobStepSeq(self.job_step_seq),
			JobStepRecordProperty::Status(self.status),
			JobStepRecordProperty::CreatedTime(self.created_time),
			JobStepRecordProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<JobStepRecordProperty> for JobStepRecord {
    fn eq(&self, property: &JobStepRecordProperty) -> bool {
        match property {
			JobStepRecordProperty::Id(id) => id == &self.id,
			JobStepRecordProperty::OrgId(org_id) => org_id == &self.org_id,
			JobStepRecordProperty::JobId(job_id) => job_id == &self.job_id,
			JobStepRecordProperty::EnvironmentId(environment_id) => environment_id == &self.environment_id,
			JobStepRecordProperty::RecordId(record_id) => record_id == &self.record_id,
			JobStepRecordProperty::JobStepId(job_step_id) => job_step_id == &self.job_step_id,
			JobStepRecordProperty::StepName(step_name) => step_name == &self.step_name,
			JobStepRecordProperty::StepType(step_type) => step_type == &self.step_type,
			JobStepRecordProperty::StepRemark(step_remark) => step_remark == &self.step_remark,
			JobStepRecordProperty::ExtensionId(extension_id) => extension_id == &self.extension_id,
			JobStepRecordProperty::OperationId(operation_id) => operation_id == &self.operation_id,
			JobStepRecordProperty::OperationName(operation_name) => operation_name == &self.operation_name,
			JobStepRecordProperty::OperationParameter(operation_parameter) => operation_parameter == &self.operation_parameter,
			JobStepRecordProperty::Attachments(attachments) => attachments == &self.attachments,
			JobStepRecordProperty::JobStepSeq(job_step_seq) => job_step_seq == &self.job_step_seq,
			JobStepRecordProperty::Status(status) => status == &self.status,
			JobStepRecordProperty::CreatedTime(created_time) => created_time == &self.created_time,
			JobStepRecordProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct JobStepRecordOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub job_id: Option<Id>,
    pub environment_id: Option<Id>,
    pub record_id: Option<Id>,
    pub job_step_id: Option<Id>,
    pub step_name: Option<String>,
    pub step_type: Option<enums::StepType>,
    pub step_remark: Option<String>,
    pub extension_id: Option<String>,
    pub operation_id: Option<String>,
    pub operation_name: Option<String>,
    pub operation_parameter: Option<String>,
    pub attachments: Option<String>,
    pub job_step_seq: Option<i32>,
    pub status: Option<enums::Status>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl JobStepRecordOpt {
    pub fn empty() -> JobStepRecordOpt {
        return JobStepRecordOpt {
            id: None,
            org_id: None,
            job_id: None,
            environment_id: None,
            record_id: None,
            job_step_id: None,
            step_name: None,
            step_type: None,
            step_remark: None,
            extension_id: None,
            operation_id: None,
            operation_name: None,
            operation_parameter: None,
            attachments: None,
            job_step_seq: None,
            status: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}