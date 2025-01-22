use crate::get_context;
use crate::middleware::auth::Guest;
use crate::model::system_user::SystemUserOpt;
use crate::native_common;
use crate::sdk;
use crate::service::base::SystemUserBaseService;
use native_common::utils::decrypt_by_base64;
use native_common::utils::encrypt_by_base64;
use native_common::utils::sha512;
use sdk::auth::calc_salt;
use sdk::auth::get_salt::GetSaltReq;
use sdk::auth::get_salt::GetSaltResp;
use sdk::auth::RandomValue;
use tihu::LightString;
use tihu_native::errno::open_transaction_error;
use tihu_native::ErrNo;

pub async fn get_salt(_guest: Guest, get_salt_req: GetSaltReq) -> Result<GetSaltResp, ErrNo> {
    let context = get_context()?;
    let mut client = context.get_db_client().await?;
    let transaction = client.transaction().await.map_err(open_transaction_error)?;
    let system_user_base_service = SystemUserBaseService::new(&transaction);
    let mut list = system_user_base_service
        .query_system_user(
            1,
            1,
            &SystemUserOpt {
                email: Some(get_salt_req.account.clone()),
                ..SystemUserOpt::empty()
            },
        )
        .await?;
    if let Some(system_user) = list.pop() {
        let user_random_value =
            decrypt_by_base64(&system_user.user_random_value).map_err(|err| {
                log::error!("解码用户随机数失败: {:?}", err);
                return ErrNo::CommonError(LightString::from_static("解码用户随机数失败！"));
            })?;
        if 32 == user_random_value.len() {
            let mut data = [0u8; 32];
            data.copy_from_slice(&user_random_value);
            let salt = calc_salt(RandomValue::Client(data), sha512).map_err(|err| {
                log::error!("计算盐值失败: {:?}", err);
                return ErrNo::CommonError(err);
            })?;
            return encrypt_by_base64(&salt).map_err(ErrNo::CommonError);
        } else {
            return Err(ErrNo::CommonError(LightString::from_static(
                "客户端随机数位数不正确！",
            )));
        }
    } else {
        let context = get_context()?;
        let server_random_value = context.get_server_random_value().await?;
        let salt = calc_salt(
            RandomValue::Server(get_salt_req.account.as_bytes(), server_random_value),
            sha512,
        )
        .map_err(|err| {
            log::error!("计算盐值失败: {:?}", err);
            return ErrNo::CommonError(err);
        })?;
        return encrypt_by_base64(&salt).map_err(ErrNo::CommonError);
    }
}
