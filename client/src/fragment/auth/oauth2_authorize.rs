use crate::sdk;
use crate::utils::request::ApiExt;
use crate::LightString;
use sdk::auth::get_curr_user::GetCurrUserResp;
use sdk::auth::login_by_oauth2_code::LoginByOauth2CodeApi;
use sdk::auth::login_by_oauth2_code::LoginByOauth2CodeReq;
use std::collections::HashMap;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub provider: String,
    pub ondone: Callback<GetCurrUserResp>,
}

#[function_component]
pub fn Oauth2Authorize(props: &Props) -> Html {
    let provider = props.provider.clone();
    let ondone = props.ondone.clone();
    use_effect_with((), move |_| {
        let ondone = ondone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            login_by_code(provider, &ondone).await.ok();
        });
        || ()
    });
    html! {}
}

async fn login_by_code(
    provider: String,
    ondone: &Callback<GetCurrUserResp>,
) -> Result<(), LightString> {
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
        .ok_or_else(|| LightString::from("No parameter \"code\" found!"))?;
    let local_storage = window.local_storage().unwrap().unwrap();
    let state = map
        .remove("state")
        .ok_or_else(|| LightString::from("No parameter \"state\" found!"))?;
    let csrf_token = local_storage
        .get(&format!("csrf_token_{}", provider))
        .unwrap();
    let csrf_token = csrf_token.ok_or_else(|| LightString::from("No csrf_token found!"))?;
    if csrf_token != state {
        return Err(LightString::from(
            "Parameter \"state\" not match, illegal access!",
        ));
    }
    let pkce_verifier = local_storage
        .get(&format!("pkce_verifier_{}", provider))
        .unwrap();
    let params = LoginByOauth2CodeReq {
        provider: provider,
        code: code,
        pkce_verifier: pkce_verifier,
    };
    let curr_user = LoginByOauth2CodeApi.call(&params).await?;
    ondone.emit(curr_user);
    return Ok(());
}
