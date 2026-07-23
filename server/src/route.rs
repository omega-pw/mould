use crate::action;
use crate::native_common;
use crate::prehandle::session::SignatureResult;
use crate::sdk;
use crate::Context;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::Request;
use native_common::errno::gen_no_such_api;
use native_common::utils::sha512;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tihu::Api;
use tihu_native::http::FromRequest;
use tihu_native::http::RequestData;
use tihu_native::ErrNo;

pub async fn dispatch(
    (context, request, remote_addr, request_data): (
        Arc<Context>,
        Request<Incoming>,
        SocketAddr,
        RequestData,
    ),
) -> Result<Bytes, ErrNo> {
    match request.uri().path() {
        //获取系统信息
        sdk::system::get_system_info::GET_SYSTEM_INFO_API => {
            call_api(
                context,
                sdk::system::get_system_info::GetSystemInfoApi,
                action::system::get_system_info::get_system_info,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //获取盐值接口
        sdk::auth::get_salt::GET_SALT_API => {
            call_api(
                context,
                sdk::auth::get_salt::GetSaltApi,
                action::auth::get_salt::get_salt,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //获取nonce接口
        sdk::auth::get_nonce::GET_NONCE_API => {
            call_api(
                context,
                sdk::auth::get_nonce::GetNonceApi,
                action::auth::get_nonce::get_nonce,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //发送邮箱验证码接口
        sdk::auth::send_email_captcha::SEND_EMAIL_CAPTCHA_API => {
            call_api(
                context,
                sdk::auth::send_email_captcha::SendEmailCaptchaApi,
                action::auth::send_email_captcha::send_email_captcha,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //登陆接口
        sdk::auth::login::LOGIN_API => {
            call_api(
                context,
                sdk::auth::login::LoginApi,
                action::auth::login::login,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //注册接口
        sdk::auth::register::REGISTER_API => {
            call_api(
                context,
                sdk::auth::register::RegisterApi,
                action::auth::register::register,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //获取openid providers
        sdk::auth::get_openid_providers::GET_OPENID_PROVIDERS_API => {
            call_api(
                context,
                sdk::auth::get_openid_providers::GetOpenidProvidersApi,
                action::auth::get_openid_providers::get_openid_providers,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //通过纯oauth2授权码登录
        sdk::auth::login_by_oauth2_code::LOGIN_BY_OAUTH2_CODE_API => {
            call_api(
                context,
                sdk::auth::login_by_oauth2_code::LoginByOauth2CodeApi,
                action::auth::login_by_oauth2_code::login_by_oauth2_code,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //通过openid授权码登录
        sdk::auth::login_by_openid_code::LOGIN_BY_OPENID_CODE_API => {
            call_api(
                context,
                sdk::auth::login_by_openid_code::LoginByOpenidCodeApi,
                action::auth::login_by_openid_code::login_by_openid_code,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //重置密码
        sdk::auth::reset_password::RESET_PASSWORD_API => {
            call_api(
                context,
                sdk::auth::reset_password::ResetPasswordApi,
                action::auth::reset_password::reset_password,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //退出登陆
        sdk::auth::logout::LOGOUT_API => {
            call_api(
                context,
                sdk::auth::logout::LogoutApi,
                action::auth::logout::get_logout_url,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //获取当前登陆用户
        sdk::auth::get_curr_user::GET_CURR_USER_API => {
            call_api(
                context,
                sdk::auth::get_curr_user::GetCurrUserApi,
                action::auth::get_curr_user::get_curr_user,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //修改密码
        sdk::auth::change_password::CHANGE_PASSWORD_API => {
            call_api(
                context,
                sdk::auth::change_password::ChangePasswordApi,
                action::auth::change_password::change_password,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //查询用户列表
        sdk::user::query_user::QUERY_USER_API => {
            call_api(
                context,
                sdk::user::query_user::QueryUserApi,
                action::user::query_user::query_user,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //获取用户详情
        sdk::user::read_user::READ_USER_API => {
            call_api(
                context,
                sdk::user::read_user::ReadUserApi,
                action::user::read_user::read_user,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //邀请用户
        sdk::user::invite_user::INVITE_USER_API => {
            call_api(
                context,
                sdk::user::invite_user::InviteUserApi,
                action::user::invite_user::invite_user,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //查询扩展列表
        sdk::extension::query_extension::QUERY_EXTENSION_API => {
            call_api(
                context,
                sdk::extension::query_extension::QueryExtensionApi,
                action::extension::query_extension::query_extension,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //测试扩展配置
        sdk::extension::test_configuration::TEST_CONFIGURATION_API => {
            call_api(
                context,
                sdk::extension::test_configuration::TestConfigurationApi,
                action::extension::test_configuration::test_configuration,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //保存环境规格
        sdk::environment_schema::save_environment_schema::SAVE_ENVIRONMENT_SCHEMA_API => {
            call_api(
                context,
                sdk::environment_schema::save_environment_schema::SaveEnvironmentSchemaApi,
                action::environment_schema::save_environment_schema::save_environment_schema,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //读取环境规格
        sdk::environment_schema::read_environment_schema::READ_ENVIRONMENT_SCHEMA_API => {
            call_api(
                context,
                sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi,
                action::environment_schema::read_environment_schema::read_environment_schema,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //删除环境规格
        sdk::environment_schema::delete_environment_schema::DELETE_ENVIRONMENT_SCHEMA_API => {
            call_api(
                context,
                sdk::environment_schema::delete_environment_schema::DeleteEnvironmentSchemaApi,
                action::environment_schema::delete_environment_schema::delete_environment_schema,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //查询环境规格
        sdk::environment_schema::query_environment_schema::QUERY_ENVIRONMENT_SCHEMA_API => {
            call_api(
                context,
                sdk::environment_schema::query_environment_schema::QueryEnvironmentSchemaApi,
                action::environment_schema::query_environment_schema::query_environment_schema,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //新增环境
        sdk::environment::insert_environment::INSERT_ENVIRONMENT_API => {
            call_api(
                context,
                sdk::environment::insert_environment::InsertEnvironmentApi,
                action::environment::insert_environment::insert_environment,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //更新环境
        sdk::environment::update_environment::UPDATE_ENVIRONMENT_API => {
            call_api(
                context,
                sdk::environment::update_environment::UpdateEnvironmentApi,
                action::environment::update_environment::update_environment,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //读取环境
        sdk::environment::read_environment::READ_ENVIRONMENT_API => {
            call_api(
                context,
                sdk::environment::read_environment::ReadEnvironmentApi,
                action::environment::read_environment::read_environment,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //删除环境
        sdk::environment::delete_environment::DELETE_ENVIRONMENT_API => {
            call_api(
                context,
                sdk::environment::delete_environment::DeleteEnvironmentApi,
                action::environment::delete_environment::delete_environment,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //查询环境
        sdk::environment::query_environment::QUERY_ENVIRONMENT_API => {
            call_api(
                context,
                sdk::environment::query_environment::QueryEnvironmentApi,
                action::environment::query_environment::query_environment,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //新增job任务
        sdk::job::insert_job::INSERT_JOB_API => {
            call_api(
                context,
                sdk::job::insert_job::InsertJobApi,
                action::job::insert_job::insert_job,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //更新job任务
        sdk::job::update_job::UPDATE_JOB_API => {
            call_api(
                context,
                sdk::job::update_job::UpdateJobApi,
                action::job::update_job::update_job,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //读取job任务
        sdk::job::read_job::READ_JOB_API => {
            call_api(
                context,
                sdk::job::read_job::ReadJobApi,
                action::job::read_job::read_job,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //删除job任务
        sdk::job::delete_job::DELETE_JOB_API => {
            call_api(
                context,
                sdk::job::delete_job::DeleteJobApi,
                action::job::delete_job::delete_job,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //查询job任务列表
        sdk::job::query_job::QUERY_JOB_API => {
            call_api(
                context,
                sdk::job::query_job::QueryJobApi,
                action::job::query_job::query_job,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //查询job任务执行记录
        sdk::job_record::query_job_record::QUERY_JOB_RECORD_API => {
            call_api(
                context,
                sdk::job_record::query_job_record::QueryJobRecordApi,
                action::job_record::query_job_record::query_job_record,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //启动job任务
        sdk::job::start_job::START_JOB_API => {
            call_api(
                context,
                sdk::job::start_job::StartJobApi,
                action::job::start_job::start_job,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //继续job任务
        sdk::job::continue_job::CONTINUE_JOB_API => {
            call_api(
                context,
                sdk::job::continue_job::ContinueJobApi,
                action::job::continue_job::continue_job,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        //读取job任务记录
        sdk::job_record::read_job_record::READ_JOB_RECORD_API => {
            call_api(
                context,
                sdk::job_record::read_job_record::ReadJobRecordApi,
                action::job_record::read_job_record::read_job_record,
                request,
                remote_addr,
                request_data,
            )
            .await
        }
        _ => Ok(gen_no_such_api().into()),
    }
}

/**
 * 获取并校验请求
 */
pub fn get_and_validate_req<I>(_api: I, req: &[u8]) -> Result<I::Input, ErrNo>
where
    I: Api,
    I::Input: DeserializeOwned,
{
    let req = serde_json::from_slice(req).map_err(|err| {
        log::error!("请求参数格式错误: {:?}", err);
        ErrNo::ParamFormatError
    })?;
    I::validate_input(&req).map_err(|err| {
        log::error!("请求参数校验失败: {:?}", err);
        ErrNo::ParamInvalid(err)
    })?;
    return Ok(req);
}

/**
 * 调用api
 */
pub async fn try_call_api<P, F, I>(
    context: Arc<Context>,
    api: I,
    handler: impl Fn(Arc<Context>, P, I::Input) -> F,
    request: Request<Incoming>,
    remote_addr: SocketAddr,
    mut request_data: RequestData,
) -> Result<I::Output, ErrNo>
where
    P: FromRequest<Arc<Context>, ErrNo>,
    F: Future<Output = Result<I::Output, ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    let signature_result = request_data
        .try_get::<SignatureResult, Arc<Context>, ErrNo>(&context, &request, remote_addr)
        .await?;
    let body_hash = signature_result.body_hash.clone();
    let prefetch_data = request_data
        .remove_or_get::<P, Arc<Context>, ErrNo>(&context, &request, remote_addr)
        .await?;
    let (parts, body) = request.into_parts();
    let route = parts.uri.path();
    let body = body
        .collect()
        .await
        .map_err(|err| ErrNo::Other(err.into()))?
        .to_bytes();
    let actual_hash = sha512(&body);
    if actual_hash.as_slice() != &body_hash {
        log::error!("请求体hash不一致: {}", route);
        return Err(ErrNo::NotAllowed);
    }
    let req = get_and_validate_req(api, &body)?;
    return handler(context, prefetch_data, req).await;
}

/**
 * 调用api
 */
pub async fn call_api<P, F, I>(
    context: Arc<Context>,
    api: I,
    handler: impl Fn(Arc<Context>, P, I::Input) -> F,
    request: Request<Incoming>,
    remote_addr: SocketAddr,
    request_data: RequestData,
) -> Result<Bytes, ErrNo>
where
    P: FromRequest<Arc<Context>, ErrNo>,
    F: Future<Output = Result<I::Output, ErrNo>>,
    I: Api,
    I::Input: DeserializeOwned,
    I::Output: Serialize,
{
    let resp = try_call_api(context, api, handler, request, remote_addr, request_data).await?;
    serde_json::to_vec(&tihu::api::Response::success(Some(resp)))
        .map(From::from)
        .map_err(ErrNo::SerializeError)
}
