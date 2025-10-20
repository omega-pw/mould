use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::SharedString;

pub const LOGOUT_API: &str = "/api/auth/logout";

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutReq {
    pub redirect_uri: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutUrl {
    pub url: String,
    pub client_id: String,
    pub id_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutResp {
    pub redirect_uri: Option<String>,
}

pub struct LogoutApi;
impl Api for LogoutApi {
    type Input = LogoutReq;
    type Output = LogoutResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(LOGOUT_API);
    }
}
