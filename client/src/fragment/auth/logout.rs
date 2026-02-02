use crate::sdk;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
use sdk::auth::logout::LogoutApi;
use sdk::auth::logout::LogoutReq;

#[component]
pub fn Logout() -> impl IntoView {
    wasm_bindgen_futures::spawn_local(async move {
        logout().await.ok();
    });
    view! {}
}

async fn logout() -> Result<(), SharedString> {
    let params = LogoutReq { redirect_uri: None };
    let resp = LogoutApi.call(&params).await?;
    let redirect_uri = resp.redirect_uri.as_deref().unwrap_or("/login");
    let window = web_sys::window().unwrap();
    window.location().assign(redirect_uri).unwrap();
    return Ok(());
}
