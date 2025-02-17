use crate::action;
use crate::middleware::auth::call_guest_api;
use crate::middleware::auth::call_user_api;
use crate::middleware::auth::Guest;
use crate::middleware::auth::User;
use crate::native_common;
use crate::sdk;
use bytes::Bytes;
use native_common::errno::gen_no_such_api;
use tihu::LightString;
use tihu_native::ErrNo;

pub async fn dispatch_user_api(
    (route, req, user): (LightString, Bytes, User),
) -> Result<Bytes, ErrNo> {
    let resp = match route.as_str() {
        //修改密码
        sdk::auth::change_password::CHANGE_PASSWORD_API => {
            call_user_api(
                sdk::auth::change_password::ChangePasswordApi,
                action::auth::change_password::change_password,
                user,
                &req,
            )
            .await
        }
        //查询用户列表
        sdk::user::query_user::QUERY_USER_API => {
            call_user_api(
                sdk::user::query_user::QueryUserApi,
                action::user::query_user::query_user,
                user,
                &req,
            )
            .await
        }
        //获取用户详情
        sdk::user::read_user::READ_USER_API => {
            call_user_api(
                sdk::user::read_user::ReadUserApi,
                action::user::read_user::read_user,
                user,
                &req,
            )
            .await
        }
        //邀请用户
        sdk::user::invite_user::INVITE_USER_API => {
            call_user_api(
                sdk::user::invite_user::InviteUserApi,
                action::user::invite_user::invite_user,
                user,
                &req,
            )
            .await
        }
        //查询扩展列表
        sdk::extension::query_extension::QUERY_EXTENSION_API => {
            call_user_api(
                sdk::extension::query_extension::QueryExtensionApi,
                action::extension::query_extension::query_extension,
                user,
                &req,
            )
            .await
        }
        //测试扩展配置
        sdk::extension::test_configuration::TEST_CONFIGURATION_API => {
            call_user_api(
                sdk::extension::test_configuration::TestConfigurationApi,
                action::extension::test_configuration::test_configuration,
                user,
                &req,
            )
            .await
        }
        //保存环境规格
        sdk::environment_schema::save_environment_schema::SAVE_ENVIRONMENT_SCHEMA_API => {
            call_user_api(
                sdk::environment_schema::save_environment_schema::SaveEnvironmentSchemaApi,
                action::environment_schema::save_environment_schema::save_environment_schema,
                user,
                &req,
            )
            .await
        }
        //读取环境规格
        sdk::environment_schema::read_environment_schema::READ_ENVIRONMENT_SCHEMA_API => {
            call_user_api(
                sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi,
                action::environment_schema::read_environment_schema::read_environment_schema,
                user,
                &req,
            )
            .await
        }
        //删除环境规格
        sdk::environment_schema::delete_environment_schema::DELETE_ENVIRONMENT_SCHEMA_API => {
            call_user_api(
                sdk::environment_schema::delete_environment_schema::DeleteEnvironmentSchemaApi,
                action::environment_schema::delete_environment_schema::delete_environment_schema,
                user,
                &req,
            )
            .await
        }
        //查询环境规格
        sdk::environment_schema::query_environment_schema::QUERY_ENVIRONMENT_SCHEMA_API => {
            call_user_api(
                sdk::environment_schema::query_environment_schema::QueryEnvironmentSchemaApi,
                action::environment_schema::query_environment_schema::query_environment_schema,
                user,
                &req,
            )
            .await
        }
        //新增环境
        sdk::environment::insert_environment::INSERT_ENVIRONMENT_API => {
            call_user_api(
                sdk::environment::insert_environment::InsertEnvironmentApi,
                action::environment::insert_environment::insert_environment,
                user,
                &req,
            )
            .await
        }
        //更新环境
        sdk::environment::update_environment::UPDATE_ENVIRONMENT_API => {
            call_user_api(
                sdk::environment::update_environment::UpdateEnvironmentApi,
                action::environment::update_environment::update_environment,
                user,
                &req,
            )
            .await
        }
        //读取环境
        sdk::environment::read_environment::READ_ENVIRONMENT_API => {
            call_user_api(
                sdk::environment::read_environment::ReadEnvironmentApi,
                action::environment::read_environment::read_environment,
                user,
                &req,
            )
            .await
        }
        //删除环境
        sdk::environment::delete_environment::DELETE_ENVIRONMENT_API => {
            call_user_api(
                sdk::environment::delete_environment::DeleteEnvironmentApi,
                action::environment::delete_environment::delete_environment,
                user,
                &req,
            )
            .await
        }
        //查询环境
        sdk::environment::query_environment::QUERY_ENVIRONMENT_API => {
            call_user_api(
                sdk::environment::query_environment::QueryEnvironmentApi,
                action::environment::query_environment::query_environment,
                user,
                &req,
            )
            .await
        }
        //新增job任务
        sdk::job::insert_job::INSERT_JOB_API => {
            call_user_api(
                sdk::job::insert_job::InsertJobApi,
                action::job::insert_job::insert_job,
                user,
                &req,
            )
            .await
        }
        //更新job任务
        sdk::job::update_job::UPDATE_JOB_API => {
            call_user_api(
                sdk::job::update_job::UpdateJobApi,
                action::job::update_job::update_job,
                user,
                &req,
            )
            .await
        }
        //读取job任务
        sdk::job::read_job::READ_JOB_API => {
            call_user_api(
                sdk::job::read_job::ReadJobApi,
                action::job::read_job::read_job,
                user,
                &req,
            )
            .await
        }
        //删除job任务
        sdk::job::delete_job::DELETE_JOB_API => {
            call_user_api(
                sdk::job::delete_job::DeleteJobApi,
                action::job::delete_job::delete_job,
                user,
                &req,
            )
            .await
        }
        //查询job任务列表
        sdk::job::query_job::QUERY_JOB_API => {
            call_user_api(
                sdk::job::query_job::QueryJobApi,
                action::job::query_job::query_job,
                user,
                &req,
            )
            .await
        }
        //查询job任务执行记录
        sdk::job_record::query_job_record::QUERY_JOB_RECORD_API => {
            call_user_api(
                sdk::job_record::query_job_record::QueryJobRecordApi,
                action::job_record::query_job_record::query_job_record,
                user,
                &req,
            )
            .await
        }
        //启动job任务
        sdk::job::start_job::START_JOB_API => {
            call_user_api(
                sdk::job::start_job::StartJobApi,
                action::job::start_job::start_job,
                user,
                &req,
            )
            .await
        }
        //继续job任务
        sdk::job::continue_job::CONTINUE_JOB_API => {
            call_user_api(
                sdk::job::continue_job::ContinueJobApi,
                action::job::continue_job::continue_job,
                user,
                &req,
            )
            .await
        }
        //读取job任务记录
        sdk::job_record::read_job_record::READ_JOB_RECORD_API => {
            call_user_api(
                sdk::job_record::read_job_record::ReadJobRecordApi,
                action::job_record::read_job_record::read_job_record,
                user,
                &req,
            )
            .await
        }
        _ => gen_no_such_api().into(),
    };
    return Ok(resp);
}

pub async fn dispatch_guest_api(
    (route, req, guest): (LightString, Bytes, Guest),
) -> Result<Bytes, ErrNo> {
    let resp = match route.as_str() {
        //获取盐值接口
        sdk::auth::get_salt::GET_SALT_API => {
            call_guest_api(
                sdk::auth::get_salt::GetSaltApi,
                action::auth::get_salt::get_salt,
                guest,
                &req,
            )
            .await
        }
        //获取公钥接口
        sdk::auth::get_rsa_pub_key::GET_RSA_PUB_KEY_API => {
            call_guest_api(
                sdk::auth::get_rsa_pub_key::GetRsaPubKeyApi,
                action::auth::get_rsa_pub_key::get_rsa_pub_key,
                guest,
                &req,
            )
            .await
        }
        //获取nonce接口
        sdk::auth::get_nonce::GET_NONCE_API => {
            call_guest_api(
                sdk::auth::get_nonce::GetNonceApi,
                action::auth::get_nonce::get_nonce,
                guest,
                &req,
            )
            .await
        }
        //发送邮箱验证码接口
        sdk::auth::send_email_captcha::SEND_EMAIL_CAPTCHA_API => {
            call_guest_api(
                sdk::auth::send_email_captcha::SendEmailCaptchaApi,
                action::auth::send_email_captcha::send_email_captcha,
                guest,
                &req,
            )
            .await
        }
        //登陆接口
        sdk::auth::login::LOGIN_API => {
            call_guest_api(
                sdk::auth::login::LoginApi,
                action::auth::login::login,
                guest,
                &req,
            )
            .await
        }
        //注册接口
        sdk::auth::register::REGISTER_API => {
            call_guest_api(
                sdk::auth::register::RegisterApi,
                action::auth::register::register,
                guest,
                &req,
            )
            .await
        }
        //获取openid providers
        sdk::auth::get_openid_providers::GET_OPENID_PROVIDERS_API => {
            call_guest_api(
                sdk::auth::get_openid_providers::GetOpenidProvidersApi,
                action::auth::get_openid_providers::get_openid_providers,
                guest,
                &req,
            )
            .await
        }
        //通过纯oauth2授权码登录
        sdk::auth::login_by_oauth2_code::LOGIN_BY_OAUTH2_CODE_API => {
            call_guest_api(
                sdk::auth::login_by_oauth2_code::LoginByOauth2CodeApi,
                action::auth::login_by_oauth2_code::login_by_oauth2_code,
                guest,
                &req,
            )
            .await
        }
        //通过openid授权码登录
        sdk::auth::login_by_openid_code::LOGIN_BY_OPENID_CODE_API => {
            call_guest_api(
                sdk::auth::login_by_openid_code::LoginByOpenidCodeApi,
                action::auth::login_by_openid_code::login_by_openid_code,
                guest,
                &req,
            )
            .await
        }
        //重置密码
        sdk::auth::reset_password::RESET_PASSWORD_API => {
            call_guest_api(
                sdk::auth::reset_password::ResetPasswordApi,
                action::auth::reset_password::reset_password,
                guest,
                &req,
            )
            .await
        }
        //退出登陆
        sdk::auth::logout::LOGOUT_API => {
            call_guest_api(
                sdk::auth::logout::LogoutApi,
                action::auth::logout::get_logout_url,
                guest,
                &req,
            )
            .await
        }
        //获取当前登陆用户
        sdk::auth::get_curr_user::GET_CURR_USER_API => {
            call_guest_api(
                sdk::auth::get_curr_user::GetCurrUserApi,
                action::auth::get_curr_user::get_curr_user,
                guest,
                &req,
            )
            .await
        }
        _ => gen_no_such_api().into(),
    };
    return Ok(resp);
}

pub static WHITE_LIST_NAMESPACE: &[&'static str] = &[
    "/blob/",
    "/file/",
    sdk::auth::get_salt::GET_SALT_API,
    sdk::auth::get_rsa_pub_key::GET_RSA_PUB_KEY_API,
    sdk::auth::get_nonce::GET_NONCE_API,
    sdk::auth::send_email_captcha::SEND_EMAIL_CAPTCHA_API,
    sdk::auth::login::LOGIN_API,
    sdk::auth::register::REGISTER_API,
    sdk::auth::get_openid_providers::GET_OPENID_PROVIDERS_API,
    sdk::auth::login_by_oauth2_code::LOGIN_BY_OAUTH2_CODE_API,
    sdk::auth::login_by_openid_code::LOGIN_BY_OPENID_CODE_API,
    sdk::auth::logout::LOGOUT_API,
    sdk::auth::get_curr_user::GET_CURR_USER_API,
];
