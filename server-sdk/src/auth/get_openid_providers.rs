use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::SharedString;

pub const GET_OPENID_PROVIDERS_API: &str = "/api/auth/getOpenidProviders";

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOpenidProvidersReq {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenidProvider {
    pub key: String,
    pub name: String,
}

pub type GetOpenidProvidersResp = Vec<OpenidProvider>;

pub struct GetOpenidProvidersApi;
impl Api for GetOpenidProvidersApi {
    type Input = GetOpenidProvidersReq;
    type Output = GetOpenidProvidersResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(GET_OPENID_PROVIDERS_API);
    }
}
