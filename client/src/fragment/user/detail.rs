use crate::components::image::Image;
use crate::sdk;
use crate::utils;
use crate::utils::format_time_local;
use crate::utils::request::ApiExt;
use crate::LightString;
use sdk::user::read_user::ReadUserApi;
use sdk::user::read_user::ReadUserReq;
use sdk::user::read_user::User;
use sdk::user::read_user::UserSource;
use tihu::Id;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Id,
}

#[function_component]
pub fn UserDetail(props: &Props) -> Html {
    let detail: UseStateHandle<Option<User>> = use_state(|| None);
    let id = props.id;
    let detail_clone = detail.clone();
    use_effect_with(id, move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            read_user_detail(&detail_clone, id).await.ok();
        });
        || ()
    });
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"名称："}</td>
                    <td>{detail.as_ref().map(|user|{html!{&user.name}}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"头像："}</td>
                    <td>{detail.as_ref().map(|user|{user.avatar_url.as_ref().map(|avatar_url|{
                        html!{
                            <Image src={LightString::from(avatar_url.clone())} style="max-height: 3em;"/>
                        }
                    }).unwrap_or_else(utils::empty_html)}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
                {
                    if let Some(user) = detail.as_ref() {
                        match &user.user_source {
                            UserSource::System(system_user) => {
                                html! {
                                    <tr>
                                        <td class="align-right" style="width:8em;vertical-align: top;">{"邮箱："}</td>
                                        <td>{&system_user.email}</td>
                                    </tr>
                                }
                            },
                            UserSource::External(external_user) => {
                                html! {
                                    <>
                                        <tr>
                                            <td class="align-right" style="width:8em;vertical-align: top;">{"提供者："}</td>
                                            <td>{&external_user.provider}</td>
                                        </tr>
                                        <tr>
                                            <td class="align-right" style="width:8em;vertical-align: top;">{"openid："}</td>
                                            <td>{&external_user.openid}</td>
                                        </tr>
                                        <tr>
                                            <td class="align-right" style="width:8em;vertical-align: top;">{"详细："}</td>
                                            <td>
                                                {
                                                    if let Some(detail) = external_user.detail.as_ref() {
                                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(detail) {
                                                            html! {render_json(&json)}
                                                        } else {
                                                            html! {&detail}
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </td>
                                        </tr>
                                    </>
                                }
                            }
                        }
                    } else {
                        html! {}
                    }
                }
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"创建时间："}</td>
                    <td>{detail.as_ref().map(|user|{{ html!{&format_time_local(&user.created_time)} }}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"更新时间："}</td>
                    <td>{detail.as_ref().map(|user|{{ html!{&format_time_local(&user.last_modified_time)} }}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
            </table>
        </div>
    }
}

async fn read_user_detail(
    detail: &UseStateHandle<Option<User>>,
    id: Id,
) -> Result<(), LightString> {
    let params = ReadUserReq { id: id };
    let user = ReadUserApi.call(&params).await?;
    detail.set(user);
    return Ok(());
}

fn render_json(json: &serde_json::Value) -> Html {
    match json {
        serde_json::Value::Null => {
            html! { "null" }
        }
        serde_json::Value::Bool(value) => {
            html! { value }
        }
        serde_json::Value::Number(value) => {
            html! { value }
        }
        serde_json::Value::String(value) => {
            html! { value }
        }
        serde_json::Value::Array(array) => {
            html! {
                <ul>
                    {
                        for array.iter().map(|value| {
                            html! {
                                <li>
                                    {render_json(value)}
                                </li>
                            }
                        })
                    }
                </ul>
            }
        }
        serde_json::Value::Object(object) => {
            html! {
                <table class="e-table">
                    {
                        for object.iter().map(|(key, value)| {
                            html! {
                                <tr class="e-table-row">
                                    <td class="e-table-cell">{key}{":"}</td>
                                    <td class="e-table-cell">{render_json(value)}</td>
                                </tr>
                            }
                        })
                    }
                </table>
            }
        }
    }
}
