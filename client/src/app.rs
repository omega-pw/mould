use crate::components::button::Button;
use crate::components::center_middle::CenterMiddle;
use crate::components::image::Image;
use crate::components::loading::Loading;
use crate::components::modal_dialog::ModalDialog;
use crate::fragment::auth::login_or_register::LoginOrRegister;
use crate::fragment::auth::logout::Logout;
use crate::fragment::auth::oauth2_authorize::Oauth2Authorize;
use crate::fragment::auth::oidc_authorize::OidcAuthorize;
use crate::fragment::auth::reset_password::ResetPassword;
use crate::fragment::change_password::ChangePassword;
use crate::fragment::environment::list::EnvironmentList;
use crate::fragment::environment_schema::list::EnvironmentSchemaList;
use crate::fragment::index::Index;
use crate::fragment::job::list::JobList;
use crate::fragment::job_record::detail::JobRecordDetail;
use crate::fragment::job_record::list::JobRecordList;
use crate::fragment::sys_menu::SysMenu;
use crate::fragment::user::list::UserList;
use crate::route::is_white_list_route;
use crate::route::Route;
use crate::sdk;
use crate::utils;
use crate::AppContext;
use crate::Context;
use crate::ContextAction;
use crate::LightString;
use sdk::auth::get_curr_user::AuthSource;
use sdk::auth::get_curr_user::GetCurrUserApi;
use sdk::auth::get_curr_user::GetCurrUserReq;
use sdk::auth::get_curr_user::GetCurrUserResp;
use utils::request::ApiExt;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct RootProps {
    pub route: Route,
}

#[function_component]
pub fn RootApp(props: &RootProps) -> Html {
    let app_context = use_context::<AppContext>().expect("no app context found");
    let inited: UseStateHandle<bool> = use_state(|| false);
    let change_password_active: UseStateHandle<bool> = use_state(|| false);
    let route = props.route.clone();
    let navigator = use_navigator().unwrap();
    let on_logout = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Logout);
        })
    };
    let inited_clone = inited.clone();
    let route_clone = route.clone();
    let app_context_clone = app_context.clone();
    use_effect_with((), move |_| {
        if is_white_list_route(&route_clone) {
            inited_clone.set(true);
        } else {
            wasm_bindgen_futures::spawn_local(async move {
                match GetCurrUserApi.call(&GetCurrUserReq {}).await {
                    Ok(curr_user) => {
                        if let Some(curr_user) = curr_user {
                            app_context_clone.dispatch(ContextAction::UpdateUser(curr_user));
                            inited_clone.set(true);
                        } else {
                            let window = web_sys::window().unwrap();
                            window.location().assign("/login").unwrap();
                        }
                    }
                    Err(err) => {
                        log::error!("Get current user failed, {}", err);
                    }
                }
            });
        }
        || ()
    });
    if *inited {
        let on_login_done = {
            let app_context = app_context.clone();
            Callback::from(move |curr_user: GetCurrUserResp| {
                if let Some(curr_user) = curr_user {
                    app_context.dispatch(ContextAction::UpdateUser(curr_user));
                    navigator.push(&Route::Index);
                } else {
                    let window = web_sys::window().unwrap();
                    window.location().assign("/").unwrap();
                }
            })
        };
        html! {
            match route {
                Route::Login => html! { <LoginOrRegister ondone={on_login_done.clone()}/> },
                Route::Oauth2Authorize { provider } => html! { <Oauth2Authorize provider={provider} ondone={on_login_done.clone()}/> },
                Route::OidcAuthorize { provider } => html! { <OidcAuthorize provider={provider} ondone={on_login_done}/> },
                Route::ResetPassword => html! { <ResetPassword/> },
                Route::Logout => html! { <Logout/> },
                _ => {
                    if let Some(curr_user) = app_context.curr_user.as_ref() {
                        html! {
                            <div class="relative width-fill height-fill">
                                <div class="absolute dock-top" style="display:flex;justify-content:space-between;align-items:center;padding-left: 0.5em;padding-right: 0.5em;height:2.5em;">
                                    <div>{"Mould"}</div>
                                    <div style="display: flex; align-items: center;">
                                        if let Some(avatar_url) = curr_user.avatar_url.as_ref() {
                                            <Image src={LightString::from(avatar_url.clone())} style="max-height: 2em;margin-right:0.5em;"/>
                                        }
                                        {curr_user.name.clone()}
                                        {
                                            match &curr_user.auth_source {
                                                AuthSource::External { .. } => {
                                                    html! {}
                                                }
                                                AuthSource::System { .. } => {
                                                    let change_password_active = change_password_active.clone();
                                                    let on_change_password = Callback::from(move |_| {
                                                        change_password_active.set(true);
                                                    });
                                                    html! {
                                                        <Button onclick={on_change_password} style="margin-left:0.5em;">{"修改密码"}</Button>
                                                    }
                                                }
                                            }
                                        }
                                        <Button onclick={on_logout} style="margin-left:0.5em;">{"退出"}</Button>
                                    </div>
                                </div>
                                <div class="absolute dock-bottom" style="border-top: 1px solid #CCC;top:2.5em;">
                                    if curr_user.org_id.is_some() {
                                        <div style="position: absolute;width: 16em;height: 100%;left: 0;box-sizing: border-box;border-right: 1px solid #CCC;">
                                            <SysMenu route={route.clone()}/>
                                        </div>
                                        <div style="position: absolute;left: 16em;height: 100%;right: 0;">
                                            {
                                                match route {
                                                    Route::Login | Route::Oauth2Authorize { .. } | Route::OidcAuthorize { .. } | Route::ResetPassword | Route::Logout => html! {},
                                                    Route::Index => html! { <Index /> },
                                                    Route::EnvironmentSchemaList => html! { <EnvironmentSchemaList /> },
                                                    Route::EnvironmentList => html! { <EnvironmentList /> },
                                                    Route::JobList => html! { <JobList /> },
                                                    Route::JobRecordListByJob { job_id } => html! { <JobRecordList job_id={job_id} /> },
                                                    Route::JobRecordListByEnvironment { environment_id } => html! { <JobRecordList environment_id={environment_id} /> },
                                                    Route::JobRecord { id } => html! { <JobRecordDetail id={id} /> },
                                                    Route::UserList => html! { <UserList /> },
                                                }
                                            }
                                        </div>
                                    } else {
                                        <CenterMiddle>
                                            {format!("你还没有加入任何组织，请联系组织成员添加，你的id：{}", curr_user.id)}
                                        </CenterMiddle>
                                    }
                                </div>
                                {
                                    if *change_password_active {
                                        let onclose = Callback::from(move |_| {
                                            change_password_active.set(false);
                                        });
                                        html! {
                                            <ModalDialog title={"修改密码"} closable=true onclose={onclose.clone()}>
                                                <ChangePassword ondone={onclose}/>
                                            </ModalDialog>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            }
        }
    } else {
        html! {
            <Loading center_middle={true}/>
        }
    }
}

#[function_component]
pub fn App() -> Html {
    let app_context: AppContext = use_reducer(|| Context::default());
    html! {
        <ContextProvider<AppContext> context={app_context}>
            <BrowserRouter>
                <Switch<Route> render={|route: Route| {
                    html! {
                        <RootApp route={route.clone()} />
                    }
                }} />
            </BrowserRouter>
        </ContextProvider<AppContext>>
    }
}
