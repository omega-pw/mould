use super::enums;
use chrono;
use chrono::DateTime;
use chrono::Utc;
use serde;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::pagination::PaginationList;
use tihu::Api;
use tihu::Id;
use tihu::LightString;

pub const QUERY_JOB_RECORD_API: &str = "/api/job/queryJobRecord";

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryJobRecordReq {
    pub job_id: Option<Id>,
    pub environment_id: Option<Id>,
    pub status: Option<enums::RecordStatus>,
    pub page_no: Option<u64>,
    pub page_size: Option<u64>,
}

impl QueryJobRecordReq {
    pub fn empty() -> QueryJobRecordReq {
        return QueryJobRecordReq {
            job_id: None,
            environment_id: None,
            status: None,
            page_no: None,
            page_size: None,
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRecord {
    pub id: Id,
    pub job_id: Id,
    pub job_name: Option<String>,
    pub environment_id: Id,
    pub environment_name: Option<String>,
    pub status: enums::RecordStatus,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

pub type QueryJobRecordResp = PaginationList<JobRecord>;

pub struct QueryJobRecordApi;
impl Api for QueryJobRecordApi {
    type Input = QueryJobRecordReq;
    type Output = QueryJobRecordResp;
    fn namespace() -> LightString {
        return LightString::from_static(QUERY_JOB_RECORD_API);
    }
}
