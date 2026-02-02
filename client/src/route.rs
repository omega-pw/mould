use leptos_router::path;
use leptos_router::{ParamSegment, StaticSegment};
// use tihu::Id;

pub static INDEX: () = path!("/");
pub static LOGIN: (StaticSegment<&str>,) = path!("/login");
pub static OAUTH2_AUTHORIZE: (StaticSegment<&str>, StaticSegment<&str>, ParamSegment) =
    path!("/oauth2/authorize/:provider");
pub static OIDC_AUTHORIZE: (StaticSegment<&str>, StaticSegment<&str>, ParamSegment) =
    path!("/oidc/authorize/:provider");
pub static RESET_PASSWORD: (StaticSegment<&str>,) = path!("/resetPassword");
pub static LOGOUT: (StaticSegment<&str>,) = path!("/logout");

pub static ENVIRONMENT_SCHEMA_LIST: (StaticSegment<&str>,) = path!("/environmentSchemaList");
pub static ENVIRONMENT_LIST: (StaticSegment<&str>,) = path!("/environmentList");
pub static JOB_LIST: (StaticSegment<&str>,) = path!("/jobList");
pub static JOB_RECORD_LIST_BY_JOB: (StaticSegment<&str>, StaticSegment<&str>, ParamSegment) =
    path!("/jobRecord/listByJob/:job_id");
pub static JOB_RECORD_LIST_BY_ENVIRONMENT: (
    StaticSegment<&str>,
    StaticSegment<&str>,
    ParamSegment,
) = path!("/jobRecord/listByEnvironment/:environment_id");
pub static JOB_RECORD: (StaticSegment<&str>, ParamSegment) = path!("/jobRecord/:id");
pub static USER_LIST: (StaticSegment<&str>,) = path!("/userList");

pub fn is_white_list_route(path: &str) -> bool {
    if ["/login", "/resetPassword", "/logout"]
        .iter()
        .any(|item| item == &path)
    {
        return true;
    } else if ["/oauth2/authorize/", "/oidc/authorize/"]
        .iter()
        .any(|item| path.starts_with(item))
    {
        return true;
    } else {
        return false;
    }
}
