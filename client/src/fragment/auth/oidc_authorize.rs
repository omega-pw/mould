use crate::sdk;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
use leptos::Params;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::login_by_openid_code::LoginByOpenidCodeApi;
use sdk::auth::login_by_openid_code::LoginByOpenidCodeReq;
use std::collections::HashMap;

#[derive(Params, Clone, PartialEq)]
struct AuthParams {
    provider: Option<String>,
}

#[component]
pub fn OidcAuthorizePage(ondone: UnsyncCallback<GetCurrUserResp>) -> impl IntoView {
    let auth_params = use_params::<AuthParams>();
    move || {
        let auth_params = auth_params.get();
        match auth_params {
            Ok(auth_params) => {
                if let Some(provider) = auth_params.provider {
                    view! {
                        <OidcAuthorize provider={provider} ondone={ondone}/>
                    }
                    .into_any()
                } else {
                    view! {
                        <div>
                            {"参数错误: provider为空"}
                        </div>
                    }
                    .into_any()
                }
            }
            Err(err) => view! {
                <div>
                    {format!("参数错误: {}", err)}
                </div>
            }
            .into_any(),
        }
    }
}

#[component]
pub fn OidcAuthorize(provider: String, ondone: UnsyncCallback<GetCurrUserResp>) -> impl IntoView {
    wasm_bindgen_futures::spawn_local(async move {
        login_by_code(provider, &ondone).await.ok();
    });
    view! {}
}

async fn login_by_code(
    provider: String,
    ondone: &UnsyncCallback<GetCurrUserResp>,
) -> Result<(), SharedString> {
    let window = web_sys::window().unwrap();
    let mut query = window.location().search().unwrap();
    if !query.is_empty() {
        query = query.split_off(1);
    }
    let mut map: HashMap<String, String> = HashMap::new();
    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        map.insert(key.to_string(), value.to_string());
    }
    let code = map
        .remove("code")
        .ok_or_else(|| SharedString::from("No parameter \"code\" found!"))?;
    let params = LoginByOpenidCodeReq {
        provider: provider,
        code: code,
    };
    let curr_user = LoginByOpenidCodeApi.call(&params).await?;
    ondone.run(curr_user);
    return Ok(());
}
