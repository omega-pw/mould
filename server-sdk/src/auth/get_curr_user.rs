use super::super::user::enums::ProviderType;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;

pub const GET_CURR_USER_API: &str = "/api/auth/getCurrUser";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCurrUserReq {}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct User {
    pub id: Id,
    pub org_id: Option<Id>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub auth_source: AuthSource,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum AuthSource {
    System {
        email: String,
        user_random_value: String,
    },
    External {
        provider_type: ProviderType,
        provider: String,
        openid: String,
        detail: String,
    },
}

pub type GetCurrUserResp = Option<User>;

pub struct GetCurrUserApi;
impl Api for GetCurrUserApi {
    type Input = GetCurrUserReq;
    type Output = GetCurrUserResp;
    fn namespace() -> LightString {
        return LightString::from_static(GET_CURR_USER_API);
    }
}
