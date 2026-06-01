use crate::cache::SYSTEM_INFO;
use crate::components::button::Button;
use crate::components::input::Input;
use crate::js;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::utils::result::ResultExt;
use crate::AppContext;
use crate::SharedString;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js::sha512;
use js::RsaPubKey2048;
use leptos::prelude::*;
use log;
use sdk::auth::calc_salt;
use sdk::auth::change_password::ChangePasswordApi;
use sdk::auth::change_password::ChangePasswordReq;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::User;
use sdk::auth::get_nonce::GetNonceApi;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::RandomValue;
use std::sync::LazyLock;

#[derive(Clone)]
struct ChangeForm {
    old_password: RwSignal<SharedString>,
    new_password: RwSignal<SharedString>,
    confirm_new_password: RwSignal<SharedString>,
}

#[component]
pub fn ChangePassword(curr_user: User, ondone: UnsyncCallback<()>) -> impl IntoView {
    let app_context = use_context::<AppContext>().expect("no app context found");
    let form = ChangeForm {
        old_password: RwSignal::new("".into()),
        new_password: RwSignal::new("".into()),
        confirm_new_password: RwSignal::new("".into()),
    };
    let is_saving: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    wasm_bindgen_futures::spawn_local(async move {
        LazyLock::force(&SYSTEM_INFO).get_fresh_data().await.ok();
    });

    let err_msg_clone = err_msg.clone();
    let clear_err_msg = UnsyncCallback::new(move |_| {
        err_msg_clone.set(None);
    });

    let is_saving_clone = is_saving.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    match &curr_user.auth_source {
        AuthSource::External { .. } => view! {}.into_any(),
        AuthSource::System {
            user_random_value, ..
        } => {
            let user_random_value = user_random_value.clone();
            let on_save = UnsyncCallback::new(move |_| {
                let is_saving = is_saving_clone.clone();
                let form = form_clone.clone();
                let err_msg = err_msg_clone.clone();
                let user_random_value = user_random_value.clone();
                let ondone = ondone.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    save_change(&user_random_value, &is_saving, &form, &err_msg, &ondone)
                        .await
                        .display_error();
                });
            });
            view! {
                <div class="width-fill height-fill border-box" style="padding:0.25em;">
                    <table style="border-collapse:collapse;table-layout: fixed;">
                        <tr>
                            <td class="align-right" style="width:8em;">{"旧密码："}</td>
                            <td>
                                <Input r#type="password" disable_trim={true} value={form.old_password.clone()} onfocus={clear_err_msg.clone()}/>
                            </td>
                        </tr>
                        <tr>
                            <td class="align-right" style="width:8em;">{"新密码："}</td>
                            <td>
                                <Input r#type="password" disable_trim={true} value={form.new_password.clone()} onfocus={clear_err_msg.clone()}/>
                            </td>
                        </tr>
                        <tr>
                            <td class="align-right" style="width:8em;">{"确认新密码："}</td>
                            <td>
                                <Input r#type="password" disable_trim={true} value={form.confirm_new_password.clone()} onfocus={clear_err_msg}/>
                            </td>
                        </tr>
                        <tr>
                            <td></td>
                            <td>
                                <Button disabled={is_saving} onclick={on_save}>{"保存"}</Button>
                                <Show
                                    when={ let err_msg = err_msg.clone(); move || { err_msg.read().is_some() } }
                                >
                                    <span class="middle" style="color:red;margin-left: 0.5em;">{err_msg}</span>
                                </Show>
                            </td>
                        </tr>
                    </table>
                </div>
            }.into_any()
        }
    }
}

fn chk_form_err(form: &ChangeForm) -> Vec<SharedString> {
    let mut err_msgs: Vec<SharedString> = Vec::new();
    let old_password = form.old_password.get();
    let old_password: &str = old_password.as_ref();
    if old_password.is_empty() {
        err_msgs.push(SharedString::from("请输入旧密码"));
    }
    let new_password = form.new_password.get();
    let new_password: &str = new_password.as_ref();
    if new_password.is_empty() {
        err_msgs.push(SharedString::from("请输入新密码"));
    }
    let confirm_new_password = form.confirm_new_password.get();
    let confirm_new_password: &str = confirm_new_password.as_ref();
    if confirm_new_password.is_empty() {
        err_msgs.push(SharedString::from("请输入确认新密码"));
    }
    if confirm_new_password != new_password {
        err_msgs.push(SharedString::from("新密码不一致"));
    }
    if old_password == new_password {
        err_msgs.push(SharedString::from("新旧密码不能相同"));
    }
    return err_msgs;
}

async fn save_change(
    user_random_value: &str,
    is_saving: &RwSignal<bool>,
    form: &ChangeForm,
    err_msg: &RwSignal<Option<SharedString>>,
    ondone: &UnsyncCallback<()>,
) -> Result<(), SharedString> {
    let err_msgs = chk_form_err(form);
    if let Some(msg) = err_msgs.first() {
        err_msg.set(Some(msg.clone()));
        return Err(msg.clone());
    }
    if is_saving.get() {
        return Ok(());
    }
    is_saving.set(true);
    let form = form.clone();
    let result = change_password(user_random_value, form).await;
    is_saving.set(false);
    match result {
        Err(err) => {
            log::error!("{}", err);
            err_msg.set(Some(err));
        }
        Ok(_) => {
            utils::success(SharedString::from("修改成功"));
            ondone.run(());
        }
    }
    return Ok(());
}

async fn change_password(user_random_value: &str, form: ChangeForm) -> Result<(), SharedString> {
    let user_random_value =
        BASE64_STANDARD
            .decode(user_random_value)
            .map_err(|err| -> SharedString {
                log::error!("解码客户端随机数失败: {:?}", err);
                return SharedString::from("解码客户端随机数失败！");
            })?;
    if 32 != user_random_value.len() {
        return Err(SharedString::from("客户端随机数位数不正确！"));
    }
    let mut data = [0u8; 32];
    data.copy_from_slice(&user_random_value);
    let salt = calc_salt(RandomValue::Client(data), sha512)
        .map_err(|error| SharedString::from(error.to_string()))?;
    let (old_auth_key, _old_encryption_key) =
        sdk::auth::calc_derived_key(form.old_password.get().as_bytes(), &salt);
    let (new_auth_key, _new_encryption_key) =
        sdk::auth::calc_derived_key(form.new_password.get().as_bytes(), &salt);
    let old_auth_key = BASE64_STANDARD.encode(&old_auth_key);
    let new_auth_key = BASE64_STANDARD.encode(&new_auth_key);
    let params = GetNonceReq {};
    let (system_info_ret, nonce_ret) = futures::future::join(
        LazyLock::force(&SYSTEM_INFO).get_fresh_data(),
        GetNonceApi.call(&params),
    )
    .await;
    let system_info = system_info_ret?;
    let nonce = nonce_ret?;
    let rsa_pub_key = RsaPubKey2048::try_from_string(system_info.rsa_pub_key.as_ref());
    let cipher_old_auth_key = rsa_pub_key
        .encrypt(&[old_auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密旧授权秘钥失败！"))?;
    let cipher_old_auth_key = BASE64_STANDARD.encode(&cipher_old_auth_key.to_vec());
    let cipher_new_auth_key = rsa_pub_key
        .encrypt(&[new_auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| SharedString::from("加密新授权秘钥失败！"))?;
    let cipher_new_auth_key = BASE64_STANDARD.encode(&cipher_new_auth_key.to_vec());
    let params = ChangePasswordReq {
        nonce: nonce.to_string(),
        old_auth_key: cipher_old_auth_key,
        new_auth_key: cipher_new_auth_key,
    };
    ChangePasswordApi.call(&params).await?;
    return Ok(());
}
