use super::super::common::turnstile::TokenResult;
use super::super::common::turnstile::Turnstile;
use crate::cache::SYSTEM_INFO;
use crate::components::button::Button;
use crate::components::center_middle::CenterMiddle;
use crate::components::input::Input;
use crate::js;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::utils::result::ResultExt;
use crate::SharedString;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js::sha512;
use js::RsaPubKey2048;
use leptos::prelude::*;
use log;
use sdk::auth::calc_salt;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::get_nonce::GetNonceApi;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::register::RegisterApi;
use sdk::auth::register::RegisterReq;
use sdk::auth::send_email_captcha::Scene;
use sdk::auth::send_email_captcha::SendEmailCaptchaApi;
use sdk::auth::send_email_captcha::SendEmailCaptchaReq;
use sdk::auth::RandomValue;
use std::sync::LazyLock;
use validator::ValidateEmail;

#[derive(Clone)]
struct RegisterForm {
    account: RwSignal<SharedString>,
    password: RwSignal<SharedString>,
    confirm_password: RwSignal<SharedString>,
    captcha: RwSignal<SharedString>,
}

#[component]
pub fn Register(ondone: UnsyncCallback<GetCurrUserResp>) -> impl IntoView {
    let turnstile_token_callback: RwSignal<Option<UnsyncCallback<TokenResult>>> =
        RwSignal::new(None);
    let form = RegisterForm {
        account: RwSignal::new("".into()),
        password: RwSignal::new("".into()),
        confirm_password: RwSignal::new("".into()),
        captcha: RwSignal::new("".into()),
    };
    let is_registering: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    wasm_bindgen_futures::spawn_local(async move {
        LazyLock::force(&SYSTEM_INFO).get_fresh_data().await.ok();
    });

    let err_msg_clone = err_msg.clone();
    let clear_err_msg = UnsyncCallback::new(move |_| {
        err_msg_clone.set(None);
    });
    let account = form.account.clone();
    let on_send_captcha = UnsyncCallback::new(move |_| {
        if turnstile_token_callback.read().is_some() {
            return;
        }
        let account = account.clone();
        let token_callback = turnstile_token_callback.clone();
        turnstile_token_callback.set(Some(UnsyncCallback::new(
            move |token_result: TokenResult| {
                //把回调清空，让人机验证的UI消失，以支持下次重新创建组件实例
                token_callback.set(None);
                let account = account.clone();
                let token_result = token_result.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match token_result {
                        TokenResult::NotRequired => {
                            start_send_captcha(account.read().to_string(), None)
                                .await
                                .ok();
                        }
                        TokenResult::Success(token) => {
                            start_send_captcha(account.read().to_string(), Some(token.to_string()))
                                .await
                                .ok();
                        }
                        TokenResult::Failure(error) => {
                            err_msg.set(Some(error));
                        }
                    }
                });
            },
        )));
    });
    let is_registering_clone = is_registering.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    let on_submit = UnsyncCallback::new(move |_| {
        let is_registering = is_registering_clone.clone();
        let form = form_clone.clone();
        let err_msg = err_msg_clone.clone();
        let ondone = ondone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_register(&is_registering, &form, &err_msg, &ondone)
                .await
                .display_error();
        });
    });
    view! {
        <CenterMiddle>
            <table style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"邮箱："}</td>
                    <td style="padding-bottom: 1em;">
                        <Input value={form.account.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"密码："}</td>
                    <td style="padding-bottom: 1em;">
                        <Input r#type="password" disable_trim={true} value={form.password.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"确认密码："}</td>
                    <td style="padding-bottom: 1em;">
                        <Input r#type="password" disable_trim={true} value={form.confirm_password.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"验证码："}</td>
                    <td style="padding-bottom: 1em;">
                        {
                            let turnstile_token_callback = turnstile_token_callback.clone();
                            move || {
                                if let Some(token_callback) = turnstile_token_callback.get() {
                                    view! {
                                        <Turnstile ondone=token_callback/>
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }
                            }
                        }
                        <div>
                            <Input value={form.captcha.clone()} onfocus={clear_err_msg} onenter={on_submit.clone()} style="width:9em;"/>
                            <Button disabled={
                                let account = form.account.clone();
                                move || {
                                    let account = account.read();
                                    let account: &str = account.as_ref();
                                    account.is_empty() || !ValidateEmail::validate_email(&account) || turnstile_token_callback.read().is_some()
                                }
                            } onclick={on_send_captcha}>{"发送验证码"}</Button>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td>
                        <Button disabled={is_registering} onclick={on_submit} style={SharedString::from("padding-left: 1em;padding-right: 1em;")}>{"注册"}</Button>
                        <Show
                            when={ let err_msg = err_msg.clone(); move || { err_msg.read().is_some() } }
                        >
                            <span class="middle" style="color:red;margin-left: 0.5em;">{err_msg}</span>
                        </Show>
                    </td>
                </tr>
            </table>
        </CenterMiddle>
    }
}

fn chk_form_err(form: &RegisterForm) -> Vec<SharedString> {
    let mut err_msgs: Vec<SharedString> = Vec::new();
    let account = form.account.get();
    let account: &str = account.as_ref();
    if account.is_empty() {
        err_msgs.push("请输入邮箱".into());
    }
    if !ValidateEmail::validate_email(&account) {
        err_msgs.push("邮箱格式不正确".into());
    }
    let password = form.password.get();
    let password: &str = password.as_ref();
    if password.is_empty() {
        err_msgs.push("请输入密码".into());
    }
    let confirm_password = form.confirm_password.get();
    let confirm_password: &str = confirm_password.as_ref();
    if confirm_password != password {
        err_msgs.push("密码不一致".into());
    }
    if form.captcha.read().is_empty() {
        err_msgs.push("请输入验证码".into());
    }
    return err_msgs;
}

async fn start_send_captcha(account: String, token: Option<String>) -> Result<(), SharedString> {
    let params = SendEmailCaptchaReq {
        token: token,
        scene: Scene::Register,
        email: account,
    };
    SendEmailCaptchaApi.call(&params).await?;
    utils::success(SharedString::from("发送成功"));
    return Ok(());
}

async fn start_register(
    is_registering: &RwSignal<bool>,
    form: &RegisterForm,
    err_msg: &RwSignal<Option<SharedString>>,
    ondone: &UnsyncCallback<GetCurrUserResp>,
) -> Result<(), SharedString> {
    let err_msgs = chk_form_err(form);
    if let Some(msg) = err_msgs.first() {
        err_msg.set(Some(msg.clone()));
        return Err(msg.clone());
    }
    if is_registering.get() {
        return Ok(());
    }
    is_registering.set(true);
    let result = register(form).await;
    is_registering.set(false);
    match result {
        Err(err) => {
            log::error!("{}", err);
            err_msg.set(Some(err));
        }
        Ok(curr_user) => {
            ondone.run(curr_user);
        }
    }
    return Ok(());
}

async fn register(form: &RegisterForm) -> Result<GetCurrUserResp, SharedString> {
    let mut user_random_value = [0u8; 32];
    utils::fill_random_bytes(&mut user_random_value);
    let params = GetNonceReq {};
    let (system_info_ret, nonce_ret) = futures::future::join(
        LazyLock::force(&SYSTEM_INFO).get_fresh_data(),
        GetNonceApi.call(&params),
    )
    .await;
    let system_info = system_info_ret?;
    let nonce = nonce_ret?;
    let server_rsa_pub_key = RsaPubKey2048::try_from_string(system_info.rsa_pub_key.as_ref());
    let cipher_account = server_rsa_pub_key
        .encrypt(&[form.account.get().as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密账户失败！"))?;
    let cipher_account = BASE64_STANDARD.encode(&cipher_account);
    let salt = calc_salt(RandomValue::Client(user_random_value), sha512)
        .map_err(|err| SharedString::from(err.to_string()))?;
    let (auth_key, _encryption_key) =
        sdk::auth::calc_derived_key(form.password.get().as_bytes(), &salt);
    let auth_key = BASE64_STANDARD.encode(&auth_key);
    let cipher_auth_key = server_rsa_pub_key
        .encrypt(&[auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密授权秘钥失败！"))?;
    let cipher_auth_key = BASE64_STANDARD.encode(&cipher_auth_key);
    let user_random_value = BASE64_STANDARD.encode(&user_random_value);
    let params = RegisterReq {
        nonce: nonce.to_string(),
        account: cipher_account,
        user_random_value: user_random_value, //随机数
        auth_key: cipher_auth_key,            //授权秘钥
        captcha: form.captcha.get().to_string(),
    };
    let curr_user = RegisterApi.call(&params).await?;
    return Ok(curr_user);
}
