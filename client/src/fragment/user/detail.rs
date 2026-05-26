use crate::components::image::Image;
use crate::sdk;
use crate::utils::format_time_local;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
use sdk::user::read_user::ReadUserApi;
use sdk::user::read_user::ReadUserReq;
use sdk::user::read_user::User;
use sdk::user::read_user::UserSource;
use tihu::Id;

#[component]
pub fn UserDetail(#[prop(optional)] id: Id) -> impl IntoView {
    let detail: RwSignal<Option<User>> = RwSignal::new(None);
    let detail_clone = detail.clone();
    wasm_bindgen_futures::spawn_local(async move {
        read_user_detail(&detail_clone, id).await.ok();
    });
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"名称："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|user| user.name.clone())
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"头像："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || {
                                detail.read().as_ref().map(|user| user.avatar_url.clone()).flatten()
                                    .map(|avatar_url|{
                                        view!{
                                            <Image src={SharedString::from(avatar_url)} style="max-height: 3em;"/>
                                        }
                                    })
                            }
                        }
                    </td>
                </tr>
                {
                    let detail = detail.clone();
                    move || {
                        if let Some(user) = detail.read().as_ref() {
                            match &user.user_source {
                                UserSource::System(system_user) => {
                                    view! {
                                        <tr>
                                            <td class="align-right" style="width:8em;vertical-align: top;">{"邮箱："}</td>
                                            <td>{system_user.email.clone()}</td>
                                        </tr>
                                    }.into_any()
                                },
                                UserSource::External(external_user) => {
                                    view! {
                                        <>
                                            <tr>
                                                <td class="align-right" style="width:8em;vertical-align: top;">{"提供者："}</td>
                                                <td>{external_user.provider.clone()}</td>
                                            </tr>
                                            <tr>
                                                <td class="align-right" style="width:8em;vertical-align: top;">{"openid："}</td>
                                                <td>{external_user.openid.clone()}</td>
                                            </tr>
                                            <tr>
                                                <td class="align-right" style="width:8em;vertical-align: top;">{"详细："}</td>
                                                <td>
                                                    {
                                                        if let Some(detail) = external_user.detail.as_ref() {
                                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(detail) {
                                                                render_json(json)
                                                            } else {
                                                                detail.clone().into_any()
                                                            }
                                                        } else {
                                                            view! {}.into_any()
                                                        }
                                                    }
                                                </td>
                                            </tr>
                                        </>
                                    }.into_any()
                                }
                            }
                        } else {
                            view! {}.into_any()
                        }
                    }
                }
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"创建时间："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|user| format_time_local(&user.created_time))
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"更新时间："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|user| format_time_local(&user.last_modified_time))
                        }
                    </td>
                </tr>
            </table>
        </div>
    }
}

async fn read_user_detail(detail: &RwSignal<Option<User>>, id: Id) -> Result<(), SharedString> {
    let params = ReadUserReq { id: id };
    let user = ReadUserApi.call(&params).await?;
    detail.set(user);
    return Ok(());
}

fn render_json(json: serde_json::Value) -> AnyView {
    match json {
        serde_json::Value::Null => "null".into_any(),
        serde_json::Value::Bool(value) => value.into_any(),
        serde_json::Value::Number(value) => value.to_string().into_any(),
        serde_json::Value::String(value) => value.into_any(),
        serde_json::Value::Array(array) => view! {
            <ul>
                <For
                    each={ move || array.clone().into_iter().enumerate() }
                    key=|(index, _item)| { *index }
                    children=move |(_index, value)| {
                        view! {
                            <li>
                                {render_json(value)}
                            </li>
                        }
                    }
                />
            </ul>
        }
        .into_any(),
        serde_json::Value::Object(object) => view! {
            <table class="e-table">
                <For
                    each={
                        let object = object.clone();
                        move || { object.clone().into_iter() }
                    }
                    key=|(key, _value)| { key.clone() }
                    children=move |(key, value)| {
                        view! {
                            <tr class="e-table-row">
                                <td class="e-table-cell">{key}{":"}</td>
                                <td class="e-table-cell">{render_json(value)}</td>
                            </tr>
                        }
                    }
                />
            </table>
        }
        .into_any(),
    }
}
