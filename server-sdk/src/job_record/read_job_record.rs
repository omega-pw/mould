use super::enums;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use tihu::datetime_format;
use tihu::Api;
use tihu::Id;
use tihu::LightString;

pub const READ_JOB_RECORD_API: &str = "/api/job/readJobRecord";

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadJobRecordReq {
    pub id: Id,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobStepRecord {
    pub id: Id,
    pub record_id: Id,
    pub job_step_id: Id,
    pub step_name: String,
    pub step_type: enums::StepType,
    pub step_remark: Option<String>,
    pub extension_id: String,
    pub operation_id: String,
    pub operation_parameter: String,
    pub attachments: Option<String>,
    pub job_step_seq: i32,
    pub status: enums::StepRecordStatus,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Error => "ERROR",
                LogLevel::Warn => "WARN",
                LogLevel::Info => "INFO",
                LogLevel::Debug => "DEBUG",
                LogLevel::Trace => "TRACE",
            }
        )?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StepResLog {
    pub time: DateTime<Utc>,
    pub level: LogLevel,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobStepResourceRecord {
    pub id: Id,
    pub record_id: Id,
    pub job_step_record_id: Id,
    pub environment_resource_id: Id,
    pub resource_name: String,
    pub extension_configuration: String,
    pub output: Option<String>,
    pub status: enums::StepResourceRecordStatus,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StepRecord {
    Auto {
        job_step_record: JobStepRecord,
        step_resource_record_list: Vec<JobStepResourceRecord>,
    },
    Manual {
        job_step_record: JobStepRecord,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobRecord {
    pub id: Id,
    pub job_id: Id,
    pub job_name: Option<String>,
    pub environment_id: Id,
    pub environment_name: Option<String>,
    pub status: enums::RecordStatus,
    pub step_record_list: Vec<StepRecord>,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

pub type ReadJobRecordResp = JobRecord;
pub struct ReadJobRecordApi;
impl Api for ReadJobRecordApi {
    type Input = ReadJobRecordReq;
    type Output = ReadJobRecordResp;
    fn namespace() -> LightString {
        return LightString::from_static(READ_JOB_RECORD_API);
    }
}
