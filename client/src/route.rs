use tihu::Id;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Index,
    #[at("/login")]
    Login,
    #[at("/oauth2/authorize/:provider")]
    Oauth2Authorize { provider: String },
    #[at("/oidc/authorize/:provider")]
    OidcAuthorize { provider: String },
    #[at("/resetPassword")]
    ResetPassword,
    #[at("/logout")]
    Logout,
    #[at("/environmentSchemaList")]
    EnvironmentSchemaList,
    #[at("/environmentList")]
    EnvironmentList,
    #[at("/jobList")]
    JobList,
    #[at("/jobRecord/listByJob/:job_id")]
    JobRecordListByJob { job_id: Id },
    #[at("/jobRecord/listByEnvironment/:environment_id")]
    JobRecordListByEnvironment { environment_id: Id },
    #[at("/jobRecord/:id")]
    JobRecord { id: Id },
    #[at("/userList")]
    UserList,
}

pub fn is_white_list_route(route: &Route) -> bool {
    match route {
        Route::Login => true,
        Route::Oauth2Authorize { .. } => true,
        Route::OidcAuthorize { .. } => true,
        Route::ResetPassword => true,
        Route::Logout => true,
        _ => false,
    }
}
