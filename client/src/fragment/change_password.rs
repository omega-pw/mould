use crate::components::button::Button;
use crate::components::input::Input;
use crate::js;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
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
use sdk::auth::get_nonce::GetNonceApi;
use sdk::auth::get_nonce::GetNonceReq;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyApi;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyReq;
use sdk::auth::RandomValue;

#[derive(Clone)]
struct ChangeForm {
    old_password: RwSignal<SharedString>,
    new_password: RwSignal<SharedString>,
    confirm_new_password: RwSignal<SharedString>,
}

#[component]
pub fn ChangePassword(ondone: UnsyncCallback<()>) -> impl IntoView {
    let app_context = use_context::<AppContext>().expect("no app context found");
    let form = ChangeForm {
        old_password: RwSignal::new("".into()),
        new_password: RwSignal::new("".into()),
        confirm_new_password: RwSignal::new("".into()),
    };
    let rsa_pub_key: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let is_saving: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);

    let rsa_pub_key_clone = rsa_pub_key.clone();
    wasm_bindgen_futures::spawn_local(async move {
        get_rsa_pub_key(&rsa_pub_key_clone).await.ok();
    });

    let err_msg_clone = err_msg.clone();
    let clear_err_msg = UnsyncCallback::new(move |_| {
        err_msg_clone.set(None);
    });

    let is_saving_clone = is_saving.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    if let Some(curr_user) = app_context.curr_user.as_ref() {
        match &curr_user.auth_source {
            AuthSource::External { .. } => view! {}.into_any(),
            AuthSource::System {
                user_random_value, ..
            } => {
                let user_random_value = user_random_value.clone();
                let on_save = UnsyncCallback::new(move |_| {
                    let rsa_pub_key = rsa_pub_key.clone();
                    let is_saving = is_saving_clone.clone();
                    let form = form_clone.clone();
                    let err_msg = err_msg_clone.clone();
                    let user_random_value = user_random_value.clone();
                    let ondone = ondone.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        save_change(
                            &rsa_pub_key,
                            &user_random_value,
                            &is_saving,
                            &form,
                            &err_msg,
                            &ondone,
                        )
                        .await;
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
    } else {
        view! {}.into_any()
    }
}

async fn get_rsa_pub_key(rsa_pub_key: &RwSignal<Option<SharedString>>) -> Result<(), SharedString> {
    let params = GetRsaPubKeyReq {};
    let pub_key = GetRsaPubKeyApi.call(&params).await?;
    rsa_pub_key.set(Some(pub_key.into()));
    return Ok(());
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
    rsa_pub_key: &RwSignal<Option<SharedString>>,
    user_random_value: &str,
    is_saving: &RwSignal<bool>,
    form: &ChangeForm,
    err_msg: &RwSignal<Option<SharedString>>,
    ondone: &UnsyncCallback<()>,
) {
    let mut err_msgs = chk_form_err(form);
    if !err_msgs.is_empty() {
        err_msgs.reverse();
        err_msg.set(err_msgs.pop());
        return;
    }
    if let Some(rsa_pub_key) = rsa_pub_key.get() {
        if is_saving.get() {
            return;
        }
        is_saving.set(true);
        let rsa_pub_key = rsa_pub_key.to_string();
        let form = form.clone();
        let result = change_password(rsa_pub_key, user_random_value, form).await;
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
    } else {
        log::error!("rsa公钥为空");
    }
}

async fn change_password(
    rsa_pub_key: String,
    user_random_value: &str,
    form: ChangeForm,
) -> Result<(), SharedString> {
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
    let nonce = GetNonceApi.call(&params).await?;
    let rsa_pub_key = RsaPubKey2048::try_from_string(&rsa_pub_key);
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
