use crate::components::button::Button;
use crate::components::center_middle::CenterMiddle;
use crate::components::input::BindingInput;
use crate::js;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::LightString;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js::sha512;
use js::RsaPubKey2048;
use log;
use sdk::auth::calc_salt;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::get_nonce::GetNonceApi;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyApi;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyReq;
use sdk::auth::register::RegisterApi;
use sdk::auth::register::RegisterReq;
use sdk::auth::send_email_captcha::Scene;
use sdk::auth::send_email_captcha::SendEmailCaptchaApi;
use sdk::auth::send_email_captcha::SendEmailCaptchaReq;
use sdk::auth::RandomValue;
use tihu::validator::ValidateEmail;
use yew::prelude::*;

#[derive(Clone)]
struct RegisterForm {
    account: UseStateHandle<LightString>,
    password: UseStateHandle<LightString>,
    confirm_password: UseStateHandle<LightString>,
    captcha: UseStateHandle<LightString>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub ondone: Callback<GetCurrUserResp>,
}

#[function_component]
pub fn Register(props: &Props) -> Html {
    let form = RegisterForm {
        account: use_state(|| "".into()),
        password: use_state(|| "".into()),
        confirm_password: use_state(|| "".into()),
        captcha: use_state(|| "".into()),
    };
    let rsa_pub_key: UseStateHandle<Option<LightString>> = use_state(|| None);
    let is_registering: UseStateHandle<bool> = use_state(|| false);
    let err_msg: UseStateHandle<Option<LightString>> = use_state(|| None);
    let rsa_pub_key_clone = rsa_pub_key.clone();
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            get_rsa_pub_key(&rsa_pub_key_clone).await.ok();
        });
        || ()
    });

    let err_msg_clone = err_msg.clone();
    let clear_err_msg = Callback::from(move |_| {
        err_msg_clone.set(None);
    });
    let account = form.account.clone();
    let on_send_captcha = Callback::from(move |_| {
        let account = account.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_send_captcha(account.to_string()).await.ok();
        });
    });
    let is_registering_clone = is_registering.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    let ondone = props.ondone.clone();
    let on_submit = Callback::from(move |_| {
        let rsa_pub_key = rsa_pub_key.clone();
        let is_registering = is_registering_clone.clone();
        let form = form_clone.clone();
        let err_msg = err_msg_clone.clone();
        let ondone = ondone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_register(&rsa_pub_key, &is_registering, &form, &err_msg, &ondone).await;
        });
    });
    html! {
        <CenterMiddle>
            <table style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"邮箱："}</td>
                    <td style="padding-bottom: 1em;">
                        <BindingInput value={form.account.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"密码："}</td>
                    <td style="padding-bottom: 1em;">
                        <BindingInput r#type="password" disable_trim={true} value={form.password.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"确认密码："}</td>
                    <td style="padding-bottom: 1em;">
                        <BindingInput r#type="password" disable_trim={true} value={form.confirm_password.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"验证码："}</td>
                    <td style="padding-bottom: 1em;">
                        <BindingInput value={form.captcha.clone()} onfocus={clear_err_msg} onenter={on_submit.clone()} style="width:9em;"/>
                        <Button onclick={on_send_captcha}>{"发送验证码"}</Button>
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td>
                        <Button disabled={*is_registering} onclick={on_submit} style="padding-left: 1em;padding-right: 1em;">{"注册"}</Button>
                        {
                            match err_msg.as_ref() {
                                Some(err_msg) => {
                                    html!{
                                        <span class="middle" style="color:red;margin-left: 0.5em;">{err_msg}</span>
                                    }
                                },
                                None => html!{}
                            }
                        }
                    </td>
                </tr>
            </table>
        </CenterMiddle>
    }
}

async fn get_rsa_pub_key(
    rsa_pub_key: &UseStateHandle<Option<LightString>>,
) -> Result<(), LightString> {
    let params = GetRsaPubKeyReq {};
    let pub_key = GetRsaPubKeyApi.call(&params).await?;
    rsa_pub_key.set(Some(pub_key.into()));
    return Ok(());
}

fn chk_form_err(form: &RegisterForm) -> Vec<LightString> {
    let mut err_msgs: Vec<LightString> = Vec::new();
    if form.account.is_empty() {
        err_msgs.push("请输入邮箱".into());
    }
    if !ValidateEmail::validate_email(&form.account.as_ref()) {
        err_msgs.push("邮箱格式不正确".into());
    }
    if form.password.is_empty() {
        err_msgs.push("请输入密码".into());
    }
    if form.confirm_password != form.password {
        err_msgs.push("密码不一致".into());
    }
    if form.captcha.is_empty() {
        err_msgs.push("请输入验证码".into());
    }
    return err_msgs;
}

async fn start_send_captcha(account: String) -> Result<(), LightString> {
    let params = SendEmailCaptchaReq {
        scene: Scene::Register,
        email: account,
    };
    SendEmailCaptchaApi.call(&params).await?;
    utils::success(LightString::from("发送成功"));
    return Ok(());
}

async fn start_register(
    rsa_pub_key: &UseStateHandle<Option<LightString>>,
    is_registering: &UseStateHandle<bool>,
    form: &RegisterForm,
    err_msg: &UseStateHandle<Option<LightString>>,
    ondone: &Callback<GetCurrUserResp>,
) {
    let mut err_msgs = chk_form_err(form);
    if !err_msgs.is_empty() {
        err_msgs.reverse();
        err_msg.set(err_msgs.pop());
        return;
    }
    if let Some(rsa_pub_key) = rsa_pub_key.as_ref() {
        if **is_registering {
            return;
        }
        is_registering.set(true);
        let rsa_pub_key = rsa_pub_key.to_string();
        let result = register(&rsa_pub_key, form).await;
        is_registering.set(false);
        match result {
            Err(err) => {
                log::error!("{}", err);
                err_msg.set(Some(err));
            }
            Ok(curr_user) => {
                ondone.emit(curr_user);
            }
        }
    } else {
        log::error!("rsa公钥为空");
    }
}

async fn register(
    server_rsa_pub_key: &str,
    form: &RegisterForm,
) -> Result<GetCurrUserResp, LightString> {
    let mut user_random_value = [0u8; 32];
    utils::fill_random_bytes(&mut user_random_value);
    let params = GetNonceReq {};
    let nonce = GetNonceApi.call(&params).await?;
    let server_rsa_pub_key = RsaPubKey2048::try_from_string(server_rsa_pub_key);
    let cipher_account = server_rsa_pub_key
        .encrypt(&[form.account.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| LightString::from("加密账户失败！"))?;
    let cipher_account = BASE64_STANDARD.encode(&cipher_account);
    let salt = calc_salt(RandomValue::Client(user_random_value), sha512)
        .map_err(|err| LightString::from(err.to_string()))?;
    let (auth_key, _encryption_key) = sdk::auth::calc_derived_key(form.password.as_bytes(), &salt);
    let auth_key = BASE64_STANDARD.encode(&auth_key);
    let cipher_auth_key = server_rsa_pub_key
        .encrypt(&[auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| LightString::from("加密授权秘钥失败！"))?;
    let cipher_auth_key = BASE64_STANDARD.encode(&cipher_auth_key);
    let user_random_value = BASE64_STANDARD.encode(&user_random_value);
    let params = RegisterReq {
        nonce: nonce.to_string(),
        account: cipher_account,
        user_random_value: user_random_value, //随机数
        auth_key: cipher_auth_key,            //授权秘钥
        captcha: form.captcha.to_string(),
    };
    let curr_user = RegisterApi.call(&params).await?;
    return Ok(curr_user);
}
