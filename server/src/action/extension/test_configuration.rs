use crate::get_context;
use crate::middleware::auth::User;
use crate::sdk;
use sdk::extension::test_configuration::TestConfigurationReq;
use sdk::extension::test_configuration::TestConfigurationResp;
use tihu::Id;
use tihu::LightString;
use tihu_native::ErrNo;

pub async fn test_configuration(
    _org_id: Id,
    _user: User,
    test_configuration_req: TestConfigurationReq,
) -> Result<TestConfigurationResp, ErrNo> {
    let TestConfigurationReq {
        extension_id,
        extension_configuration,
    } = test_configuration_req;
    let context = get_context()?;
    let extension = context
        .get_extension(&extension_id)
        .ok_or_else(|| -> ErrNo {
            ErrNo::CommonError(LightString::from(format!(
                "id为\"{}\"的扩展未找到!",
                extension_id,
            )))
        })?;
    let extension_configuration = serde_json::from_str::<serde_json::Value>(
        &extension_configuration,
    )
    .map_err(|err| -> ErrNo {
        log::error!("扩展配置格式不正确：{}", err);
        return ErrNo::CommonError(LightString::Static("扩展配置格式不正确"));
    })?;
    extension
        .test_configuration(extension_configuration, context.get_extension_context())
        .await
        .map_err(|err| ErrNo::CommonError(err.into()))?;
    return Ok(());
}
