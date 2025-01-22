use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;

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
    fn namespace() -> LightString {
        return LightString::from_static(GET_OPENID_PROVIDERS_API);
    }
}
