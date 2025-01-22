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

pub const QUERY_JOB_API: &str = "/api/job/queryJob";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub id: Id,
    pub environment_schema_id: Id,
    pub environment_schema_name: String,
    pub name: String,
    pub remark: Option<String>,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryJobReq {
    pub name: Option<String>,
    pub page_no: Option<u64>,
    pub page_size: Option<u64>,
}

impl QueryJobReq {
    pub fn empty() -> QueryJobReq {
        return QueryJobReq {
            name: None,
            page_no: None,
            page_size: None,
        };
    }
}

pub type QueryJobResp = PaginationList<Job>;

pub struct QueryJobApi;
impl Api for QueryJobApi {
    type Input = QueryJobReq;
    type Output = QueryJobResp;
    fn namespace() -> LightString {
        return LightString::from_static(QUERY_JOB_API);
    }
}
