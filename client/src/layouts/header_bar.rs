use super::change_password::ChangePassword;
use crate::components::button::Button;
use crate::components::image::Image;
use crate::components::modal_dialog::ModalDialog;
use crate::sdk;
use crate::SharedString;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::User;

#[component]
pub fn HeaderBar(curr_user: User, onexit: UnsyncCallback<()>) -> impl IntoView {
    let navigate = use_navigate();
    let change_password_active: RwSignal<bool> = RwSignal::new(false);
    let on_logout = {
        let navigate = navigate.clone();
        UnsyncCallback::new(move |_| {
            navigate("/logout", Default::default());
            onexit.run(());
        })
    };
    view! {
        <div>
            <div style="display: flex; align-items: center;justify-content: flex-end;">
                {
                    if let Some(avatar_url) = curr_user.avatar_url.as_ref() {
                        view! {
                            <Image src={SharedString::from(avatar_url.clone())} style="max-height: 2em;margin-right:0.5em;"/>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }
                {curr_user.name.clone()}
                {
                    match &curr_user.auth_source {
                        AuthSource::External { .. } => {
                            view! {}.into_any()
                        }
                        AuthSource::System { .. } => {
                            let change_password_active = change_password_active.clone();
                            let on_change_password = UnsyncCallback::new(move |_| {
                                change_password_active.set(true);
                            });
                            view! {
                                <Button onclick={on_change_password} style={SharedString::from("margin-left:0.5em;")}>{"修改密码"}</Button>
                            }.into_any()
                        }
                    }
                }
                <Button onclick={on_logout} style={SharedString::from("margin-left:0.5em;")}>{"退出"}</Button>
            </div>
            <Show
                when=move || { change_password_active.get() }
            >
                {
                    let onclose = UnsyncCallback::new(move |_| {
                        change_password_active.set(false);
                    });
                    view! {
                        <ModalDialog clone:curr_user title={SharedString::from("修改密码")} closable=true onclose={onclose.clone()}>
                            <ChangePassword curr_user={curr_user} ondone={onclose}/>
                        </ModalDialog>
                    }
                }
            </Show>
        </div>
    }
}
