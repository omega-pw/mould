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
    pub const NAME: &str = "name";
    pub const STEP_TYPE: &str = "step_type";
    pub const SCHEMA_RESOURCE_ID: &str = "schema_resource_id";
    pub const OPERATION_ID: &str = "operation_id";
    pub const OPERATION_NAME: &str = "operation_name";
    pub const OPERATION_PARAMETER: &str = "operation_parameter";
    pub const ATTACHMENTS: &str = "attachments";
    pub const REMARK: &str = "remark";
    pub const SEQ: &str = "seq";
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
}


/**
 * 任务步骤列
 */
pub enum JobStepProperty {
    Id(Id),
    OrgId(Id),
    JobId(Id),
    Name(String),
    StepType(enums::StepType),
    SchemaResourceId(Option<Id>),
    OperationId(String),
    OperationName(String),
    OperationParameter(String),
    Attachments(Option<String>),
    Remark(Option<String>),
    Seq(i32),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for JobStepProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			JobStepProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepProperty::JobId(_) => PropertyDefine {
                key: LightString::from_static(properties::JOB_ID),
                value_type: PropertyType::Id,
				required: true,
            },
			JobStepProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepProperty::StepType(_) => PropertyDefine {
                key: LightString::from_static(properties::STEP_TYPE),
                value_type: PropertyType::Enum,
				required: true,
            },
			JobStepProperty::SchemaResourceId(_) => PropertyDefine {
                key: LightString::from_static(properties::SCHEMA_RESOURCE_ID),
                value_type: PropertyType::Id,
				required: false,
            },
			JobStepProperty::OperationId(_) => PropertyDefine {
                key: LightString::from_static(properties::OPERATION_ID),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepProperty::OperationName(_) => PropertyDefine {
                key: LightString::from_static(properties::OPERATION_NAME),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepProperty::OperationParameter(_) => PropertyDefine {
                key: LightString::from_static(properties::OPERATION_PARAMETER),
                value_type: PropertyType::String,
				required: true,
            },
			JobStepProperty::Attachments(_) => PropertyDefine {
                key: LightString::from_static(properties::ATTACHMENTS),
                value_type: PropertyType::String,
				required: false,
            },
			JobStepProperty::Remark(_) => PropertyDefine {
                key: LightString::from_static(properties::REMARK),
                value_type: PropertyType::String,
				required: false,
            },
			JobStepProperty::Seq(_) => PropertyDefine {
                key: LightString::from_static(properties::SEQ),
                value_type: PropertyType::Integer,
				required: true,
            },
			JobStepProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			JobStepProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 任务步骤
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct JobStep {
    pub id: Id, //步骤id
    pub org_id: Id, //组织id
    pub job_id: Id, //任务id
    pub name: String, //步骤名称
    pub step_type: enums::StepType, //步骤类型
    pub schema_resource_id: Option<Id>, //环境规格资源id
    pub operation_id: String, //操作id
    pub operation_name: String, //操作名称
    pub operation_parameter: String, //操作参数
    pub attachments: Option<String>, //附件
    pub remark: Option<String>, //备注
    pub seq: i32, //执行顺序
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl JobStep {
    pub fn into_properties(self) -> Vec<JobStepProperty> {
        return vec![
			JobStepProperty::Id(self.id),
			JobStepProperty::OrgId(self.org_id),
			JobStepProperty::JobId(self.job_id),
			JobStepProperty::Name(self.name),
			JobStepProperty::StepType(self.step_type),
			JobStepProperty::SchemaResourceId(self.schema_resource_id),
			JobStepProperty::OperationId(self.operation_id),
			JobStepProperty::OperationName(self.operation_name),
			JobStepProperty::OperationParameter(self.operation_parameter),
			JobStepProperty::Attachments(self.attachments),
			JobStepProperty::Remark(self.remark),
			JobStepProperty::Seq(self.seq),
			JobStepProperty::CreatedTime(self.created_time),
			JobStepProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<JobStepProperty> for JobStep {
    fn eq(&self, property: &JobStepProperty) -> bool {
        match property {
			JobStepProperty::Id(id) => id == &self.id,
			JobStepProperty::OrgId(org_id) => org_id == &self.org_id,
			JobStepProperty::JobId(job_id) => job_id == &self.job_id,
			JobStepProperty::Name(name) => name == &self.name,
			JobStepProperty::StepType(step_type) => step_type == &self.step_type,
			JobStepProperty::SchemaResourceId(schema_resource_id) => schema_resource_id == &self.schema_resource_id,
			JobStepProperty::OperationId(operation_id) => operation_id == &self.operation_id,
			JobStepProperty::OperationName(operation_name) => operation_name == &self.operation_name,
			JobStepProperty::OperationParameter(operation_parameter) => operation_parameter == &self.operation_parameter,
			JobStepProperty::Attachments(attachments) => attachments == &self.attachments,
			JobStepProperty::Remark(remark) => remark == &self.remark,
			JobStepProperty::Seq(seq) => seq == &self.seq,
			JobStepProperty::CreatedTime(created_time) => created_time == &self.created_time,
			JobStepProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct JobStepOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub job_id: Option<Id>,
    pub name: Option<String>,
    pub step_type: Option<enums::StepType>,
    pub schema_resource_id: Option<Id>,
    pub operation_id: Option<String>,
    pub operation_name: Option<String>,
    pub operation_parameter: Option<String>,
    pub attachments: Option<String>,
    pub remark: Option<String>,
    pub seq: Option<i32>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl JobStepOpt {
    pub fn empty() -> JobStepOpt {
        return JobStepOpt {
            id: None,
            org_id: None,
            job_id: None,
            name: None,
            step_type: None,
            schema_resource_id: None,
            operation_id: None,
            operation_name: None,
            operation_parameter: None,
            attachments: None,
            remark: None,
            seq: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}