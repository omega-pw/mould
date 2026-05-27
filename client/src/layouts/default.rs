use crate::components::button::Button;
use crate::components::center_middle::CenterMiddle;
use crate::components::image::Image;
use crate::components::modal_dialog::ModalDialog;
use crate::fragment::change_password::ChangePassword;
use crate::fragment::sys_menu::SysMenu;
use crate::sdk;
use crate::AppContext;
use crate::SharedString;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::nested_router::Outlet;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::User;

#[component]
pub fn DefaultLayout(
    curr_user: RwSignal<Option<User>>,
    onexit: UnsyncCallback<()>,
) -> impl IntoView {
    let app_context = use_context::<AppContext>().expect("no app context found");
    let navigate = use_navigate();
    let change_password_active: RwSignal<bool> = RwSignal::new(false);
    let on_logout = {
        let navigate = navigate.clone();
        UnsyncCallback::new(move |_| {
            navigate("/logout", Default::default());
            onexit.run(());
        })
    };
    move || {
        if let Some(curr_user) = curr_user.get() {
            view! {
                <div class="relative width-fill height-fill">
                    <div class="absolute dock-top" style="display:flex;justify-content:space-between;align-items:center;padding-left: 0.5em;padding-right: 0.5em;height:2.5em;">
                        <div>{"Mould"}</div>
                        <div style="display: flex; align-items: center;">
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
                    </div>
                    <div class="absolute dock-bottom" style="border-top: 1px solid #CCC;top:2.5em;">
                        {
                            if curr_user.org_id.is_some() {
                                view! {
                                    <div style="position: absolute;width: 16em;height: 100%;left: 0;box-sizing: border-box;border-right: 1px solid #CCC;">
                                        <SysMenu/>
                                    </div>
                                    <div style="position: absolute;left: 16em;height: 100%;right: 0;">
                                        <Outlet/>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <CenterMiddle>
                                        {format!("你还没有加入任何组织，请联系组织成员添加，你的id：{}", curr_user.id)}
                                    </CenterMiddle>
                                }.into_any()
                            }
                        }
                    </div>
                    <Show
                        when=move || { change_password_active.get() }
                    >
                        {
                            let onclose = UnsyncCallback::new(move |_| {
                                change_password_active.set(false);
                            });
                            view! {
                                <ModalDialog title={SharedString::from("修改密码")} closable=true onclose={onclose.clone()}>
                                    <ChangePassword ondone={onclose}/>
                                </ModalDialog>
                            }
                        }
                    </Show>
                </div>
            }
            .into_any()
        } else {
            view! {}.into_any()
        }
    }
}
