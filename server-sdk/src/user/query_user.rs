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

pub const QUERY_USER_API: &str = "/api/user/queryUser";

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryUserReq {
    pub id: Option<Id>,
    pub user_source: Option<enums::UserSource>,
    pub name: Option<String>,
    pub page_no: Option<u64>,
    pub page_size: Option<u64>,
}

impl QueryUserReq {
    pub fn empty() -> QueryUserReq {
        return QueryUserReq {
            id: None,
            user_source: None,
            name: None,
            page_no: None,
            page_size: None,
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Id,
    pub user_source: enums::UserSource,
    pub name: String,
    pub avatar_url: Option<String>,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

pub type QueryUserResp = PaginationList<User>;

pub struct QueryUserApi;
impl Api for QueryUserApi {
    type Input = QueryUserReq;
    type Output = QueryUserResp;
    fn namespace() -> LightString {
        return LightString::from_static(QUERY_USER_API);
    }
}
