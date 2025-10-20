use super::Extension;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::SharedString;

pub const QUERY_EXTENSION_API: &str = "/api/extension/queryExtension";

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryExtensionReq {}

pub type QueryExtensionResp = Vec<Extension>;

pub struct QueryExtensionApi;
impl Api for QueryExtensionApi {
    type Input = QueryExtensionReq;
    type Output = QueryExtensionResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(QUERY_EXTENSION_API);
    }
}
