use super::super::common::turnstile::TokenResult;
use super::super::common::turnstile::Turnstile;
use crate::components::button::Button;
use crate::components::input::Input;
use crate::components::modal_dialog::ModalDialog;
use crate::js;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::SharedString;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js::RsaPubKey2048;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use log;
use sdk::auth::get_nonce::GetNonceApi;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyApi;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyReq;
use sdk::auth::get_salt::GetSaltApi;
use sdk::auth::get_salt::GetSaltReq;
use sdk::auth::reset_password::ResetPasswordApi;
use sdk::auth::reset_password::ResetPasswordReq;
use sdk::auth::reset_password::ResetPasswordResp;
use sdk::auth::send_email_captcha::Scene;
use sdk::auth::send_email_captcha::SendEmailCaptchaApi;
use sdk::auth::send_email_captcha::SendEmailCaptchaReq;
use validator::ValidateEmail;

#[derive(Clone)]
struct ResetPasswordForm {
    account: RwSignal<SharedString>,
    password: RwSignal<SharedString>,
    confirm_password: RwSignal<SharedString>,
    captcha: RwSignal<SharedString>,
}

#[component]
pub fn ResetPassword() -> impl IntoView {
    let navigate = use_navigate();
    let token_result: RwSignal<Option<TokenResult>> = RwSignal::new(None);
    let form = ResetPasswordForm {
        account: RwSignal::new("".into()),
        password: RwSignal::new("".into()),
        confirm_password: RwSignal::new("".into()),
        captcha: RwSignal::new("".into()),
    };
    let rsa_pub_key: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let is_resetting: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let rsa_pub_key_clone = rsa_pub_key.clone();
    wasm_bindgen_futures::spawn_local(async move {
        get_rsa_pub_key(&rsa_pub_key_clone).await.ok();
    });

    let err_msg_clone = err_msg.clone();
    let clear_err_msg = UnsyncCallback::new(move |_| {
        err_msg_clone.set(None);
    });
    let account = form.account.clone();
    let on_send_captcha = UnsyncCallback::new(move |_| {
        let account = account.clone();
        let token_result = token_result.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(token_result) = token_result.get() {
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
            }
        });
    });
    let is_resetting_clone = is_resetting.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    let on_submit = UnsyncCallback::new(move |_| {
        let rsa_pub_key = rsa_pub_key.clone();
        let is_resetting = is_resetting_clone.clone();
        let form = form_clone.clone();
        let err_msg = err_msg_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_reset(&rsa_pub_key, &is_resetting, &form, &err_msg).await;
        });
    });
    let on_switch_login = move |_| {
        navigate("/login", Default::default());
    };
    view! {
        <ModalDialog title={SharedString::from("重置密码")} closable=false>
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
                        <Turnstile ondone={
                            let token_result = token_result.clone();
                            UnsyncCallback::new(move |result| {
                                token_result.set(Some(result));
                            })
                        }/>
                        <div>
                            <Input value={form.captcha.clone()} onfocus={clear_err_msg} onenter={on_submit.clone()} style="width:9em;"/>
                            <Button disabled={move || token_result.get().is_none()} onclick={on_send_captcha}>{"发送验证码"}</Button>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td>
                        <Button disabled={is_resetting} onclick={on_submit} style={SharedString::from("padding-left: 1em;padding-right: 1em;")}>{"提交"}</Button>
                        <Show
                            when={ let err_msg = err_msg.clone(); move || { err_msg.read().is_some() } }
                        >
                            <span class="middle" style="color:red;margin-left: 0.5em;">{err_msg}</span>
                        </Show>
                    </td>
                </tr>
            </table>
            <div style="text-align:right;">
                <a href="javascript:void(0);" on:click={on_switch_login}>{"登录"}</a>
            </div>
        </ModalDialog>
    }
}

async fn get_rsa_pub_key(rsa_pub_key: &RwSignal<Option<SharedString>>) -> Result<(), SharedString> {
    let params = GetRsaPubKeyReq {};
    let pub_key = GetRsaPubKeyApi.call(&params).await?;
    rsa_pub_key.set(Some(pub_key.into()));
    return Ok(());
}

fn chk_form_err(form: &ResetPasswordForm) -> Vec<SharedString> {
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
        scene: Scene::ResetPassword,
        email: account,
    };
    SendEmailCaptchaApi.call(&params).await?;
    utils::success(SharedString::from("发送成功"));
    return Ok(());
}

async fn start_reset(
    rsa_pub_key: &RwSignal<Option<SharedString>>,
    is_resetting: &RwSignal<bool>,
    form: &ResetPasswordForm,
    err_msg: &RwSignal<Option<SharedString>>,
) {
    let mut err_msgs = chk_form_err(form);
    if !err_msgs.is_empty() {
        err_msgs.reverse();
        err_msg.set(err_msgs.pop());
        return;
    }
    if let Some(rsa_pub_key) = rsa_pub_key.get() {
        if is_resetting.get() {
            return;
        }
        is_resetting.set(true);
        let rsa_pub_key = rsa_pub_key.to_string();
        let result = reset(&rsa_pub_key, form).await;
        is_resetting.set(false);
        match result {
            Err(err) => {
                log::error!("{}", err);
                err_msg.set(Some(err));
            }
            Ok(_) => {
                clear_form(form);
                utils::success(SharedString::from("重置成功"));
            }
        }
    } else {
        log::error!("rsa公钥为空");
    }
}

fn clear_form(form: &ResetPasswordForm) {
    form.account.set(Default::default());
    form.password.set(Default::default());
    form.confirm_password.set(Default::default());
    form.captcha.set(Default::default());
}

async fn reset(
    server_rsa_pub_key: &str,
    form: &ResetPasswordForm,
) -> Result<ResetPasswordResp, SharedString> {
    let account = form.account.get();
    let salt = GetSaltApi
        .call(&GetSaltReq {
            account: account.to_string(),
        })
        .await?;
    let salt = BASE64_STANDARD
        .decode(&salt)
        .map_err(|err| -> SharedString {
            log::error!("解码盐值失败: {:?}", err);
            return "解码盐值失败！".into();
        })?;
    let params = GetNonceReq {};
    let nonce = GetNonceApi.call(&params).await?;
    let server_rsa_pub_key = RsaPubKey2048::try_from_string(server_rsa_pub_key);
    let cipher_account = server_rsa_pub_key
        .encrypt(&[account.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密账户失败！"))?;
    let cipher_account = BASE64_STANDARD.encode(&cipher_account);
    let (auth_key, _encryption_key) =
        sdk::auth::calc_derived_key(form.password.get().as_bytes(), &salt);
    let auth_key = BASE64_STANDARD.encode(&auth_key);
    let cipher_auth_key = server_rsa_pub_key
        .encrypt(&[auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密授权秘钥失败！"))?;
    let cipher_auth_key = BASE64_STANDARD.encode(&cipher_auth_key.to_vec());
    let params = ResetPasswordReq {
        nonce: nonce.to_string(),
        account: cipher_account,
        auth_key: cipher_auth_key, //授权秘钥
        captcha: form.captcha.get().to_string(),
    };
    ResetPasswordApi.call(&params).await?;
    return Ok(());
}
