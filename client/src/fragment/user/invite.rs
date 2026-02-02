use crate::components::button::Button;
use crate::components::input::Input;
use crate::components::validate_wrapper::ValidateData;
use crate::components::validate_wrapper::ValidateWrapper;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::SharedString;
use leptos::prelude::*;
use sdk::user::invite_user::InviteUserApi;
use sdk::user::invite_user::InviteUserReq;
use tihu::PrimaryKey;
use uuid::Uuid;

#[derive(Clone)]
struct InviteForm {
    user_id: ValidateData<SharedString>,
}

#[component]
pub fn InviteEdit(#[prop(optional)] onsave: Option<UnsyncCallback<PrimaryKey>>) -> impl IntoView {
    let is_saving: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let invite_form = InviteForm {
        user_id: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请填写用户id"))),
        ),
    };
    let invite_form_clone = invite_form.clone();
    let is_saving_clone = is_saving.clone();
    let err_msg_clone = err_msg.clone();
    let on_save = UnsyncCallback::new(move |_| {
        let invite_form: InviteForm = invite_form_clone.clone();
        let is_saving = is_saving_clone.clone();
        let err_msg = err_msg_clone.clone();
        let onsave = onsave.clone();
        wasm_bindgen_futures::spawn_local(async move {
            save_user(&invite_form, is_saving, &err_msg, &onsave)
                .await
                .ok();
        });
    });
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><span style="color:red;margin-right: 0.25em;">{"*"}</span>{"用户id："}</td>
                    <td>
                        <ValidateWrapper error={invite_form.user_id.error()}>
                            <Input value={invite_form.user_id.data()} onupdate={invite_form.user_id.listener()}/>
                        </ValidateWrapper>
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
    }
}

fn chk_form_err(invite_form: &InviteForm) -> Vec<SharedString> {
    let mut err_msgs: Vec<SharedString> = Vec::new();
    if let Err(error) = invite_form.user_id.validate(true) {
        err_msgs.push(error);
    }
    if let Err(_err) = Uuid::parse_str(invite_form.user_id.get().as_ref()) {
        err_msgs.push(SharedString::from("请填写正确的用户id"));
    }
    return err_msgs;
}

async fn save_user(
    invite_form: &InviteForm,
    is_saving: RwSignal<bool>,
    err_msg: &RwSignal<Option<SharedString>>,
    onsave: &Option<UnsyncCallback<PrimaryKey>>,
) -> Result<(), SharedString> {
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
                onsave.run(PrimaryKey { id: user_id });
            }
            utils::success(SharedString::from("保存成功"));
        }
    }
    return Ok(());
}
