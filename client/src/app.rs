use crate::components::loading::Loading;
use crate::fragment::auth::login_or_register::LoginOrRegisterPage;
use crate::fragment::auth::logout::Logout;
use crate::fragment::auth::oauth2_authorize::Oauth2AuthorizePage;
use crate::fragment::auth::oidc_authorize::OidcAuthorizePage;
use crate::fragment::auth::reset_password::ResetPassword;
use crate::fragment::environment::list::EnvironmentList;
use crate::fragment::environment_schema::list::EnvironmentSchemaList;
use crate::fragment::index::Index;
use crate::fragment::job::list::JobList;
use crate::fragment::job_record::detail::JobRecordDetailPage;
use crate::fragment::job_record::list::JobRecordListByEnvironment;
use crate::fragment::job_record::list::JobRecordListByJob;
use crate::fragment::user::list::UserList;
use crate::layouts::DefaultLayout;
use crate::route::is_white_list_route;
use crate::route::ENVIRONMENT_LIST;
use crate::route::ENVIRONMENT_SCHEMA_LIST;
use crate::route::INDEX;
use crate::route::JOB_LIST;
use crate::route::JOB_RECORD;
use crate::route::JOB_RECORD_LIST_BY_ENVIRONMENT;
use crate::route::JOB_RECORD_LIST_BY_JOB;
use crate::route::LOGIN;
use crate::route::LOGOUT;
use crate::route::OAUTH2_AUTHORIZE;
use crate::route::OIDC_AUTHORIZE;
use crate::route::RESET_PASSWORD;
use crate::route::USER_LIST;
use crate::sdk;
use crate::utils;
use crate::AppContext;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use leptos_router::path;
use sdk::auth::get_curr_user::GetCurrUserApi;
use sdk::auth::get_curr_user::GetCurrUserReq;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::get_curr_user::User;
use utils::request::ApiExt;

#[component]
pub fn RootApp() -> impl IntoView {
    let app_context = use_context::<AppContext>().expect("no app context found");
    let navigate = use_navigate();
    let curr_user: RwSignal<Option<User>> = RwSignal::new(None);
    let inited: RwSignal<bool> = RwSignal::new(false);
    let inited_clone = inited.clone();
    let window = web_sys::window().unwrap();
    if is_white_list_route(&window.location().pathname().unwrap()) {
        inited_clone.set(true);
    } else {
        let curr_user = curr_user.clone();
        let app_context = app_context.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match GetCurrUserApi.call(&GetCurrUserReq {}).await {
                Ok(user) => {
                    if let Some(user) = user {
                        curr_user.set(Some(user));
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
    let on_exit = UnsyncCallback::new({
        let curr_user = curr_user.clone();
        move |_: ()| {
            curr_user.set(None);
        }
    });
    let on_login_done = {
        let app_context = app_context.clone();
        let curr_user = curr_user.clone();
        let navigate = navigate.clone();
        UnsyncCallback::new(move |user: GetCurrUserResp| {
            if let Some(user) = user {
                curr_user.set(Some(user));
                navigate("/", Default::default());
            } else {
                let window = web_sys::window().unwrap();
                window.location().assign("/").unwrap();
            }
        })
    };
    view! {
        <Show
            when=move || { inited.get() }
            fallback=|| view!{ <Loading center_middle={true} /> }
        >
            <Routes fallback=|| view!{ <Redirect path="/"/> }>
                <Route path=LOGIN view=move || view! { <LoginOrRegisterPage ondone={on_login_done.clone()}/> }/>
                <Route path=OAUTH2_AUTHORIZE view=move || view! { <Oauth2AuthorizePage ondone={on_login_done.clone()}/> }/>
                <Route path=OIDC_AUTHORIZE view=move || view! { <OidcAuthorizePage ondone={on_login_done}/> }/>
                <Route path=RESET_PASSWORD view=move || view! { <ResetPassword/> }/>
                <Route path=LOGOUT view=move || view! { <Logout/> }/>
                <ParentRoute path=INDEX view=move || view! { <DefaultLayout curr_user={curr_user} onexit={on_exit.clone()}/> }>
                    <Route path=path!("") view=move || view! { <Index/> }/>
                    <Route path=ENVIRONMENT_SCHEMA_LIST view=move || view! { <EnvironmentSchemaList/> }/>
                    <Route path=ENVIRONMENT_LIST view=move || view! { <EnvironmentList /> }/>
                    <Route path=JOB_LIST view=move || view! { <JobList /> }/>
                    <Route path=JOB_RECORD_LIST_BY_JOB view=move || view! { <JobRecordListByJob /> }/>
                    <Route path=JOB_RECORD_LIST_BY_ENVIRONMENT view=move || view! { <JobRecordListByEnvironment /> }/>
                    <Route path=JOB_RECORD view=move || view! { <JobRecordDetailPage /> }/>
                    <Route path=USER_LIST view=move || view! { <UserList /> }/>
                </ParentRoute>
            </Routes>
        </Show>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let app_context: AppContext = AppContext::default();
    provide_context(app_context);
    view! {
        <Router>
            <RootApp />
        </Router>
    }
}
