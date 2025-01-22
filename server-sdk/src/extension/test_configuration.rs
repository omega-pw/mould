use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::LightString;

pub const TEST_CONFIGURATION_API: &str = "/api/extension/testConfiguration";

#[derive(Serialize, Deserialize, Debug)]
pub struct TestConfigurationReq {
    pub extension_id: String,
    pub extension_configuration: String,
}

pub type TestConfigurationResp = ();
pub struct TestConfigurationApi;
impl Api for TestConfigurationApi {
    type Input = TestConfigurationReq;
    type Output = TestConfigurationResp;
    fn namespace() -> LightString {
        return LightString::from_static(TEST_CONFIGURATION_API);
    }
}
