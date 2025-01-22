use crate::sdk;
use crate::utils::request::ApiExt;
use crate::LightString;
use sdk::auth::logout::LogoutApi;
use sdk::auth::logout::LogoutReq;
use yew::prelude::*;

#[function_component]
pub fn Logout() -> Html {
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            logout().await.ok();
        });
        || ()
    });
    html! {}
}

async fn logout() -> Result<(), LightString> {
    let params = LogoutReq { redirect_uri: None };
    let resp = LogoutApi.call(&params).await?;
    let redirect_uri = resp.redirect_uri.as_deref().unwrap_or("/login");
    let window = web_sys::window().unwrap();
    window.location().assign(redirect_uri).unwrap();
    return Ok(());
}
