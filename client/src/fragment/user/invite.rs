use crate::components::button::Button;
use crate::components::input::BindingInput;
use crate::components::validate_wrapper::ValidateData;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::LightString;
use sdk::user::invite_user::InviteUserApi;
use sdk::user::invite_user::InviteUserReq;
use tihu::PrimaryKey;
use uuid::Uuid;
use yew::prelude::*;

#[derive(Clone)]
struct InviteForm {
    user_id: ValidateData<LightString>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub onsave: Option<Callback<PrimaryKey>>,
}

#[function_component]
pub fn InviteEdit(props: &Props) -> Html {
    let is_saving: UseStateHandle<bool> = use_state(|| false);
    let err_msg: UseStateHandle<Option<LightString>> = use_state(|| None);
    let invite_form = InviteForm {
        user_id: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请填写用户id"))),
        ),
    };
    let invite_form_clone = invite_form.clone();
    let is_saving_clone = is_saving.clone();
    let err_msg_clone = err_msg.clone();
    let onsave_clone = props.onsave.clone();
    let on_save = Callback::from(move |_| {
        let invite_form: InviteForm = invite_form_clone.clone();
        let is_saving = is_saving_clone.clone();
        let err_msg = err_msg_clone.clone();
        let onsave = onsave_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            save_user(&invite_form, is_saving, &err_msg, &onsave)
                .await
                .ok();
        });
    });
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><span style="color:red;margin-right: 0.25em;">{"*"}</span>{"用户id："}</td>
                    <td>
                        {
                            invite_form.user_id.view(move |user_id: UseStateHandle<LightString>, validator| {
                                html! {
                                    <BindingInput value={user_id} onupdate={validator}/>
                                }
                            })
                        }
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

fn chk_form_err(invite_form: &InviteForm) -> Vec<LightString> {
    let mut err_msgs: Vec<LightString> = Vec::new();
    if let Err(error) = invite_form.user_id.validate(true) {
        err_msgs.push(error);
    }
    if let Err(_err) = Uuid::parse_str(invite_form.user_id.get().as_ref()) {
        err_msgs.push(LightString::from("请填写正确的用户id"));
    }
    return err_msgs;
}

async fn save_user(
    invite_form: &InviteForm,
    is_saving: UseStateHandle<bool>,
    err_msg: &UseStateHandle<Option<LightString>>,
    onsave: &Option<Callback<PrimaryKey>>,
) -> Result<(), LightString> {
    let err_msgs = chk_form_err(invite_form);
    if let Some(first) = err_msgs.first() {
        err_msg.set(Some(first.clone()));
        return Err(first.clone());
    }
    let user_id = Uuid::parse_str(invite_form.user_id.get().as_ref()).unwrap();
    let params = InviteUserReq { user_id: user_id };
    let ret = InviteUserApi.lock_handler(is_saving).call(&params).await;
    match ret {
        Err(err) => {
            log::error!("{}", err);
            err_msg.set(Some(err));
        }
        Ok(_) => {
            if let Some(onsave) = onsave {
                onsave.emit(PrimaryKey { id: user_id });
            }
            utils::success(LightString::from("保存成功"));
        }
    }
    return Ok(());
}
