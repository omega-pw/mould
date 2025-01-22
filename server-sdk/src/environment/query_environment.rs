use chrono::DateTime;
use chrono::Utc;
use serde;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::pagination::PaginationList;
use tihu::Api;
use tihu::Id;
use tihu::LightString;

pub const QUERY_ENVIRONMENT_API: &str = "/api/environment/queryEnvironment";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Environment {
    pub id: Id,
    pub environment_schema_id: Id,
    pub environment_schema_name: String,
    pub name: String,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryEnvironmentReq {
    pub environment_schema_id: Option<Id>,
    pub name: Option<String>,
    pub page_no: Option<u64>,
    pub page_size: Option<u64>,
}

impl QueryEnvironmentReq {
    pub fn empty() -> QueryEnvironmentReq {
        return QueryEnvironmentReq {
            environment_schema_id: None,
            name: None,
            page_no: None,
            page_size: None,
        };
    }
}

pub type QueryEnvironmentResp = PaginationList<Environment>;

pub struct QueryEnvironmentApi;
impl Api for QueryEnvironmentApi {
    type Input = QueryEnvironmentReq;
    type Output = QueryEnvironmentResp;
    fn namespace() -> LightString {
        return LightString::from_static(QUERY_ENVIRONMENT_API);
    }
}
