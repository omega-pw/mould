use super::enums;
use chrono;
use chrono::DateTime;
use chrono::Utc;
use serde;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::Api;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;

pub const READ_USER_API: &str = "/api/user/readUser";

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemUser {
    pub id: Id,
    pub email: String,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalUser {
    pub id: Id,
    pub provider_type: enums::ProviderType,
    pub provider: String,
    pub openid: String,
    pub detail: Option<String>,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserSource {
    System(SystemUser),     //系统用户
    External(ExternalUser), //外部用户
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Id,
    pub name: String,
    pub avatar_url: Option<String>,
    pub user_source: UserSource,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

pub type ReadUserReq = PrimaryKey;
pub type ReadUserResp = Option<User>;
pub struct ReadUserApi;
impl Api for ReadUserApi {
    type Input = ReadUserReq;
    type Output = ReadUserResp;
    fn namespace() -> LightString {
        return LightString::from_static(READ_USER_API);
    }
}
