use crate::assets;
use crate::cache::SYSTEM_INFO;
use crate::components::button::Button;
use crate::components::center_middle::CenterMiddle;
use crate::components::input::Input;
use crate::js;
use crate::sdk;
use crate::utils::request::ApiExt;
use crate::utils::result::ResultExt;
use crate::SharedString;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js::RsaPubKey2048;
use leptos::prelude::*;
use log;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::get_nonce::GetNonceApi;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::get_openid_providers::GetOpenidProvidersApi;
use sdk::auth::get_openid_providers::GetOpenidProvidersReq;
use sdk::auth::get_openid_providers::OpenidProvider;
use sdk::auth::get_salt::GetSaltApi;
use sdk::auth::get_salt::GetSaltReq;
use sdk::auth::login::LoginApi;
use sdk::auth::login::LoginReq;
use sdk::auth::login::LoginResp;
use std::sync::LazyLock;

#[derive(Clone)]
struct LoginForm {
    account: RwSignal<SharedString>,
    password: RwSignal<SharedString>,
}

#[component]
pub fn Login(ondone: UnsyncCallback<GetCurrUserResp>) -> impl IntoView {
    let form = LoginForm {
        account: RwSignal::new("".into()),
        password: RwSignal::new("".into()),
    };
    let is_logining: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let openid_providers: RwSignal<Vec<OpenidProvider>> = RwSignal::new(Default::default());
    // let on_wechat = UnsyncCallback::new(move |_| {
    //     let window = web_sys::window().unwrap();
    //     window.location().assign("/oauth2/login/wechat").unwrap();
    // });
    let openid_providers_clone = openid_providers.clone();
    wasm_bindgen_futures::spawn_local(async move {
        LazyLock::force(&SYSTEM_INFO).get_fresh_data().await.ok();
    });
    wasm_bindgen_futures::spawn_local(async move {
        get_openid_providers(&openid_providers_clone).await.ok();
    });
    let err_msg_clone = err_msg.clone();
    let clear_err_msg = UnsyncCallback::new(move |_| {
        err_msg_clone.set(None);
    });
    let is_logining_clone = is_logining.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    let on_submit = UnsyncCallback::new(move |_| {
        let is_logining = is_logining_clone.clone();
        let form = form_clone.clone();
        let err_msg = err_msg_clone.clone();
        let ondone = ondone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_login(&is_logining, &form, &err_msg, &ondone)
                .await
                .display_error();
        });
    });
    let on_github = move |_| {
        let window = web_sys::window().unwrap();
        window.location().assign("/oauth2/login/github").unwrap();
    };
    view! {
        <CenterMiddle>
            <div style="text-align:right;margin-bottom:1em;">
                <img src={assets::GITHUB_LOGO.path()} on:click={on_github} style="cursor: pointer;"/>
                // <a href="javascript:void(0);" onclick={on_wechat} style="margin-left:0.5em;">{"微信登录"}</a>
                <For
                    each={
                        let openid_providers = openid_providers.clone();
                        move || { openid_providers.get().into_iter() }
                    }
                    key=|openid_provider| { openid_provider.key.clone() }
                    children=move |openid_provider| {
                        let key = &openid_provider.key;
                        let name = &openid_provider.name;
                        let path = format!("/oidc/login/{key}");
                        let button_text = format!("用{name}登陆");
                        let on_openid = move |_| {
                            let window = web_sys::window().unwrap();
                            window.location().assign(&path).unwrap();
                        };
                        view! {
                            <a href="javascript:void(0);" style="margin-left:0.5em;" on:click={on_openid}>{button_text}</a>
                        }
                    }
                />
            </div>
            <table style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"邮箱："}</td>
                    <td style="padding-bottom: 1em;">
                        <Input value={form.account.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()} tabindex={1}/>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:6em;padding-bottom: 1em;">{"密码："}</td>
                    <td style="padding-bottom: 1em;">
                        <Input r#type="password" disable_trim={true} value={form.password.clone()} onfocus={clear_err_msg.clone()} onenter={on_submit.clone()} tabindex={2}/>
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td>
                        <Button disabled={is_logining} onclick={on_submit} style={SharedString::from("padding-left: 1em;padding-right: 1em;")}>{"登录"}</Button>
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

fn chk_form_err(form: &LoginForm) -> Vec<SharedString> {
    let mut err_msgs: Vec<SharedString> = Vec::new();
    if form.account.read().is_empty() {
        err_msgs.push("请输入邮箱".into());
    }
    if form.password.read().is_empty() {
        err_msgs.push("请输入密码".into());
    }
    return err_msgs;
}

async fn start_login(
    is_logining: &RwSignal<bool>,
    form: &LoginForm,
    err_msg: &RwSignal<Option<SharedString>>,
    ondone: &UnsyncCallback<GetCurrUserResp>,
) -> Result<(), SharedString> {
    let err_msgs = chk_form_err(form);
    if let Some(msg) = err_msgs.first() {
        err_msg.set(Some(msg.clone()));
        return Err(msg.clone());
    }
    if is_logining.get() {
        return Ok(());
    }
    is_logining.set(true);
    let ret = login(form).await;
    is_logining.set(false);
    match ret {
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

async fn login(form: &LoginForm) -> Result<LoginResp, SharedString> {
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
    let (auth_key, _encryption_key) =
        sdk::auth::calc_derived_key(form.password.get().as_bytes(), &salt);
    let auth_key = BASE64_STANDARD.encode(&auth_key);
    let params = GetNonceReq {};
    let (system_info_ret, nonce_ret) = futures::future::join(
        LazyLock::force(&SYSTEM_INFO).get_fresh_data(),
        GetNonceApi.call(&params),
    )
    .await;
    let system_info = system_info_ret?;
    let nonce = nonce_ret?;
    let rsa_pub_key = RsaPubKey2048::try_from_string(system_info.rsa_pub_key.as_ref());
    let cipher_account = rsa_pub_key
        .encrypt(&[account.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密账户失败！"))?;
    let cipher_account = BASE64_STANDARD.encode(&cipher_account.to_vec());
    let cipher_auth_key = rsa_pub_key
        .encrypt(&[auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密授权秘钥失败！"))?;
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
    openid_providers: &RwSignal<Vec<OpenidProvider>>,
) -> Result<Vec<OpenidProvider>, SharedString> {
    let result = GetOpenidProvidersApi
        .call(&GetOpenidProvidersReq {})
        .await?;
    openid_providers.set(result.clone());
    return Ok(result);
}
