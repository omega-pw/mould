use crate::assets;
use crate::components::button::Button;
use crate::components::center_middle::CenterMiddle;
use crate::components::input::BindingInput;
use crate::js;
use crate::sdk;
use crate::utils::request::ApiExt;
use crate::LightString;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js::RsaPubKey2048;
use log;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::get_nonce::GetNonceApi;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::get_openid_providers::GetOpenidProvidersApi;
use sdk::auth::get_openid_providers::GetOpenidProvidersReq;
use sdk::auth::get_openid_providers::OpenidProvider;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyApi;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyReq;
use sdk::auth::get_salt::GetSaltApi;
use sdk::auth::get_salt::GetSaltReq;
use sdk::auth::login::LoginApi;
use sdk::auth::login::LoginReq;
use sdk::auth::login::LoginResp;
use yew::prelude::*;

#[derive(Clone)]
struct LoginForm {
    account: UseStateHandle<LightString>,
    password: UseStateHandle<LightString>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub ondone: Callback<GetCurrUserResp>,
}

#[function_component]
pub fn Login(props: &Props) -> Html {
    let form = LoginForm {
        account: use_state(|| "".into()),
        password: use_state(|| "".into()),
    };
    let rsa_pub_key: UseStateHandle<Option<LightString>> = use_state(|| None);
    let is_logining: UseStateHandle<bool> = use_state(|| false);
    let err_msg: UseStateHandle<Option<LightString>> = use_state(|| None);
    let rsa_pub_key_clone = rsa_pub_key.clone();
    let openid_providers: UseStateHandle<Vec<OpenidProvider>> = use_state(Default::default);
    // let on_wechat = Callback::from(move |_| {
    //     let window = web_sys::window().unwrap();
    //     window.location().assign("/oauth2/login/wechat").unwrap();
    // });
    let openid_providers_clone = openid_providers.clone();
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            get_rsa_pub_key(&rsa_pub_key_clone).await.ok();
        });
        wasm_bindgen_futures::spawn_local(async move {
            get_openid_providers(&openid_providers_clone).await.ok();
        });
        || ()
    });
    let err_msg_clone = err_msg.clone();
    let clear_err_msg = Callback::from(move |_| {
        err_msg_clone.set(None);
    });
    let is_logining_clone = is_logining.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    let ondone = props.ondone.clone();
    let on_submit = Callback::from(move |_| {
        let rsa_pub_key = rsa_pub_key.clone();
        let is_logining = is_logining_clone.clone();
        let form = form_clone.clone();
        let err_msg = err_msg_clone.clone();
        let ondone = ondone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_login(&rsa_pub_key, &is_logining, &form, &err_msg, &ondone).await;
        });
    });
    let on_github = Callback::from(move |_| {
        let window = web_sys::window().unwrap();
        window.location().assign("/oauth2/login/github").unwrap();
    });
    html! {
        <CenterMiddle>
            <div style="text-align:right;margin-bottom:1em;">
                <img src={assets::GITHUB_LOGO.path()} onclick={on_github} style="cursor: pointer;"/>
                // <a href="javascript:void(0);" onclick={on_wechat} style="margin-left:0.5em;">{"微信登录"}</a>
                {for openid_providers.iter().map(|openid_provider| {
                    let key = &openid_provider.key;
                    let name = &openid_provider.name;
                    let path = format!("/oidc/login/{key}");
                    let button_text = format!("用{name}登陆");
                    let on_openid = Callback::from(move |_| {
                        let window = web_sys::window().unwrap();
                        window.location().assign(&path).unwrap();
                    });
                    html! {
                        <a href="javascript:void(0);" style="margin-left:0.5em;" onclick={on_openid}>{button_text}</a>
                    }
                })}
            </div>
            <table style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"邮箱："}</td>
                    <td style="padding-bottom: 1em;">
                        <BindingInput value={form.account.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()} tabindex={1}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"密码："}</td>
                    <td style="padding-bottom: 1em;">
                        <BindingInput r#type="password" disable_trim={true} value={form.password.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()} tabindex={2}/>
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td>
                        <Button disabled={*is_logining} onclick={on_submit} style="padding-left: 1em;padding-right: 1em;">{"登录"}</Button>
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

fn chk_form_err(form: &LoginForm) -> Vec<LightString> {
    let mut err_msgs: Vec<LightString> = Vec::new();
    if form.account.is_empty() {
        err_msgs.push("请输入邮箱".into());
    }
    if form.password.is_empty() {
        err_msgs.push("请输入密码".into());
    }
    return err_msgs;
}

async fn start_login(
    rsa_pub_key: &UseStateHandle<Option<LightString>>,
    is_logining: &UseStateHandle<bool>,
    form: &LoginForm,
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
        if **is_logining {
            return;
        }
        is_logining.set(true);
        let ret = login(rsa_pub_key, form).await;
        is_logining.set(false);
        match ret {
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

async fn login(rsa_pub_key: &str, form: &LoginForm) -> Result<LoginResp, LightString> {
    let salt = GetSaltApi
        .call(&GetSaltReq {
            account: form.account.to_string(),
        })
        .await?;
    let salt = BASE64_STANDARD
        .decode(&salt)
        .map_err(|err| -> LightString {
            log::error!("解码盐值失败: {:?}", err);
            return "解码盐值失败！".into();
        })?;
    let (auth_key, _encryption_key) = sdk::auth::calc_derived_key(form.password.as_bytes(), &salt);
    let auth_key = BASE64_STANDARD.encode(&auth_key);
    let params = GetNonceReq {};
    let nonce = GetNonceApi.call(&params).await?;
    let rsa_pub_key = RsaPubKey2048::try_from_string(rsa_pub_key);
    let cipher_account = rsa_pub_key
        .encrypt(&[form.account.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| LightString::from("加密账户失败！"))?;
    let cipher_account = BASE64_STANDARD.encode(&cipher_account.to_vec());
    let cipher_auth_key = rsa_pub_key
        .encrypt(&[auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| LightString::from("加密授权秘钥失败！"))?;
    let cipher_auth_key = BASE64_STANDARD.encode(&cipher_auth_key.to_vec());
    let params = LoginReq {
        nonce: nonce,
        account: cipher_account,
        auth_key: cipher_auth_key,
    };
    let curr_operator = LoginApi.call(&params).await?;
    return Ok(curr_operator);
}

async fn get_openid_providers(
    openid_providers: &UseStateHandle<Vec<OpenidProvider>>,
) -> Result<Vec<OpenidProvider>, LightString> {
    let result = GetOpenidProvidersApi
        .call(&GetOpenidProvidersReq {})
        .await?;
    openid_providers.set(result.clone());
    return Ok(result);
}
