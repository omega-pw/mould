use crate::components::button::Button;
use crate::components::input::BindingInput;
use crate::js;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::AppContext;
use crate::LightString;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js::sha512;
use js::RsaPubKey2048;
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
use yew::prelude::*;

#[derive(Clone)]
struct ChangeForm {
    old_password: UseStateHandle<LightString>,
    new_password: UseStateHandle<LightString>,
    confirm_new_password: UseStateHandle<LightString>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub ondone: Callback<()>,
}

#[function_component]
pub fn ChangePassword(props: &Props) -> Html {
    let app_context = use_context::<AppContext>().expect("no app context found");
    let form = ChangeForm {
        old_password: use_state(|| "".into()),
        new_password: use_state(|| "".into()),
        confirm_new_password: use_state(|| "".into()),
    };
    let rsa_pub_key: UseStateHandle<Option<LightString>> = use_state(|| None);
    let is_saving: UseStateHandle<bool> = use_state(|| false);
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

    let is_saving_clone = is_saving.clone();
    let form_clone = form.clone();
    let err_msg_clone = err_msg.clone();
    if let Some(curr_user) = app_context.curr_user.as_ref() {
        match &curr_user.auth_source {
            AuthSource::External { .. } => {
                html! {}
            }
            AuthSource::System {
                user_random_value, ..
            } => {
                let user_random_value = user_random_value.clone();
                let ondone = props.ondone.clone();
                let on_save = Callback::from(move |_| {
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
                html! {
                    <div class="width-fill height-fill border-box" style="padding:0.25em;">
                        <table style="border-collapse:collapse;table-layout: fixed;">
                            <tr>
                                <td class="align-right" style="width:8em;">{"旧密码："}</td>
                                <td>
                                    <BindingInput r#type="password" disable_trim={true} value={form.old_password.clone()} onfocus={clear_err_msg.clone()}/>
                                </td>
                            </tr>
                            <tr>
                                <td class="align-right" style="width:8em;">{"新密码："}</td>
                                <td>
                                    <BindingInput r#type="password" disable_trim={true} value={form.new_password.clone()} onfocus={clear_err_msg.clone()}/>
                                </td>
                            </tr>
                            <tr>
                                <td class="align-right" style="width:8em;">{"确认新密码："}</td>
                                <td>
                                    <BindingInput r#type="password" disable_trim={true} value={form.confirm_new_password.clone()} onfocus={clear_err_msg}/>
                                </td>
                            </tr>
                            <tr>
                                <td></td>
                                <td>
                                    <Button disabled={*is_saving} onclick={on_save}>{"保存"}</Button>
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
                    </div>
                }
            }
        }
    } else {
        html! {}
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

fn chk_form_err(form: &ChangeForm) -> Vec<LightString> {
    let mut err_msgs: Vec<LightString> = Vec::new();
    if form.old_password.is_empty() {
        err_msgs.push(LightString::Static("请输入旧密码"));
    }
    if form.new_password.is_empty() {
        err_msgs.push(LightString::Static("请输入新密码"));
    }
    if form.confirm_new_password.is_empty() {
        err_msgs.push(LightString::Static("请输入确认新密码"));
    }
    if form.confirm_new_password != form.new_password {
        err_msgs.push(LightString::Static("新密码不一致"));
    }
    if form.old_password == form.new_password {
        err_msgs.push(LightString::Static("新旧密码不能相同"));
    }
    return err_msgs;
}

async fn save_change(
    rsa_pub_key: &UseStateHandle<Option<LightString>>,
    user_random_value: &str,
    is_saving: &UseStateHandle<bool>,
    form: &ChangeForm,
    err_msg: &UseStateHandle<Option<LightString>>,
    ondone: &Callback<()>,
) {
    let mut err_msgs = chk_form_err(form);
    if !err_msgs.is_empty() {
        err_msgs.reverse();
        err_msg.set(err_msgs.pop());
        return;
    }
    if let Some(rsa_pub_key) = rsa_pub_key.as_ref() {
        if **is_saving {
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
                utils::success(LightString::from("修改成功"));
                ondone.emit(());
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
) -> Result<(), LightString> {
    let user_random_value =
        BASE64_STANDARD
            .decode(user_random_value)
            .map_err(|err| -> LightString {
                log::error!("解码客户端随机数失败: {:?}", err);
                return LightString::Static("解码客户端随机数失败！");
            })?;
    if 32 != user_random_value.len() {
        return Err(LightString::Static("客户端随机数位数不正确！"));
    }
    let mut data = [0u8; 32];
    data.copy_from_slice(&user_random_value);
    let salt = calc_salt(RandomValue::Client(data), sha512)
        .map_err(|error| LightString::from(error.to_string()))?;
    let (old_auth_key, _old_encryption_key) =
        sdk::auth::calc_derived_key(form.old_password.as_bytes(), &salt);
    let (new_auth_key, _new_encryption_key) =
        sdk::auth::calc_derived_key(form.new_password.as_bytes(), &salt);
    let old_auth_key = BASE64_STANDARD.encode(&old_auth_key);
    let new_auth_key = BASE64_STANDARD.encode(&new_auth_key);
    let params = GetNonceReq {};
    let nonce = GetNonceApi.call(&params).await?;
    let rsa_pub_key = RsaPubKey2048::try_from_string(&rsa_pub_key);
    let cipher_old_auth_key = rsa_pub_key
        .encrypt(&[old_auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| LightString::from("加密旧授权秘钥失败！"))?;
    let cipher_old_auth_key = BASE64_STANDARD.encode(&cipher_old_auth_key.to_vec());
    let cipher_new_auth_key = rsa_pub_key
        .encrypt(&[new_auth_key.as_bytes(), nonce.as_bytes()].concat())
        .ok_or_else(|| LightString::from("加密新授权秘钥失败！"))?;
    let cipher_new_auth_key = BASE64_STANDARD.encode(&cipher_new_auth_key.to_vec());
    let params = ChangePasswordReq {
        nonce: nonce.to_string(),
        old_auth_key: cipher_old_auth_key,
        new_auth_key: cipher_new_auth_key,
    };
    ChangePasswordApi.call(&params).await?;
    return Ok(());
}
