use super::EnvironmentSchema;
use serde;
use serde::{Deserialize, Serialize};
use tihu::pagination::PaginationList;
use tihu::Api;
use tihu::LightString;

pub const QUERY_ENVIRONMENT_SCHEMA_API: &str = "/api/environmentSchema/queryEnvironmentSchema";

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryEnvironmentSchemaReq {
    pub name: Option<String>,
    pub page_no: Option<u64>,
    pub page_size: Option<u64>,
}

impl QueryEnvironmentSchemaReq {
    pub fn empty() -> QueryEnvironmentSchemaReq {
        return QueryEnvironmentSchemaReq {
            name: None,
            page_no: None,
            page_size: None,
        };
    }
}

pub type QueryEnvironmentSchemaResp = PaginationList<EnvironmentSchema>;

pub struct QueryEnvironmentSchemaApi;
impl Api for QueryEnvironmentSchemaApi {
    type Input = QueryEnvironmentSchemaReq;
    type Output = QueryEnvironmentSchemaResp;
    fn namespace() -> LightString {
        return LightString::from_static(QUERY_ENVIRONMENT_SCHEMA_API);
    }
}
