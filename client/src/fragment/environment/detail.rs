use super::super::extension::config_detail_view;
use super::super::extension::get_configuration_schema;
use super::super::extension::parse_config;
use crate::components::show::Show;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::LightString;
use sdk::environment::read_environment::Environment;
use sdk::environment::read_environment::ReadEnvironmentApi;
use sdk::environment::read_environment::ReadEnvironmentReq;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::extension::query_extension::QueryExtensionApi;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::Extension;
use std::ops::Deref;
use tihu::Id;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Id,
}

#[function_component]
pub fn EnvironmentDetail(props: &Props) -> Html {
    let active_schema_resource_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let active_resource_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let detail: UseStateHandle<Option<Environment>> = use_state(|| None);
    let environment_schema_detail: UseStateHandle<Option<EnvironmentSchema>> = use_state(|| None);
    let extension_list: UseStateHandle<Vec<Extension>> = use_state(|| Default::default());
    let id = props.id;
    let extension_list_clone = extension_list.clone();
    let detail_clone = detail.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    use_effect_with(id, move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            match query_extension_list(&extension_list_clone).await {
                Ok(_extension_list) => {
                    if let Some(environment) = read_environment_detail(&detail_clone, id).await.ok()
                    {
                        read_environment_schema_detail(
                            &environment_schema_detail_clone,
                            environment.environment_schema_id,
                        )
                        .await
                        .ok();
                    }
                }
                Err(_err) => {
                    //
                }
            }
        });
        || ()
    });
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"环境名称："}</td>
                    <td>{detail.as_ref().map(|environment|{html!{&environment.name}}).unwrap_or_else(utils::empty_html)}</td>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"环境规格："}</td>
                    <td>{environment_schema_detail.as_ref().map(|environment_schema|{html!{&environment_schema.name}}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:16em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源规格"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        {
                            if let Some(detail) = detail.as_ref() {
                                html! {
                                    for detail.schema_resource_list.iter().map(|schema_resource| {
                                        let schema_resource_name = schema_resource.name.clone();
                                        let schema_resource_id = schema_resource.id;
                                        let extension_id = schema_resource.extension_id.clone();
                                        let extension_list = extension_list.clone();
                                        let active_schema_resource_id = active_schema_resource_id.clone();
                                        let is_active = active_schema_resource_id.deref() == &Some(schema_resource_id);
                                        let background_color = if is_active {
                                            "background-color: #EEE"
                                        } else {
                                            ""
                                        };
                                        html! {
                                            <div>
                                                <div onclick={Callback::from(move |_| {
                                                    let active_schema_resource_id = active_schema_resource_id.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        active_schema_resource_id.set(Some(schema_resource_id));
                                                        utils::wait(0).await;
                                                        utils::trigger_resize();
                                                    });
                                                })} style={format!("border-bottom: 1px solid #CCC;padding: 0.5em;{}", background_color)}>
                                                    {schema_resource_name}
                                                </div>
                                                <Show condition={is_active} style="position:absolute;left:16em;right:0;top:0;bottom:0;overflow: auto;">
                                                    {
                                                        {
                                                            let active_resource_id = active_resource_id.clone();
                                                            html! {
                                                                <div style="width:20em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                                                                    <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源列表"}</div>
                                                                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                                                        {
                                                                            for schema_resource.resource_list.deref().into_iter().map(|resource| {
                                                                                let active_resource_id = active_resource_id.clone();
                                                                                let resource_id = resource.id;
                                                                                let is_active = active_resource_id.deref() == &Some(resource_id);
                                                                                let background_color = if is_active {
                                                                                    "background-color: #EEE"
                                                                                } else {
                                                                                    ""
                                                                                };
                                                                                let configuration_schema =
                                                                                get_configuration_schema(&extension_list, &extension_id)
                                                                                    .map(|configuration_schema| configuration_schema.clone())
                                                                                    .unwrap_or_default();
                                                                                let extension_configuration = parse_config(
                                                                                    configuration_schema.clone(),
                                                                                    &resource.extension_configuration,
                                                                                );
                                                                                html! {
                                                                                    <div>
                                                                                        <div onclick={Callback::from(move |_| {
                                                                                            let active_resource_id = active_resource_id.clone();
                                                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                                                active_resource_id.set(Some(resource_id));
                                                                                                utils::wait(0).await;
                                                                                                utils::trigger_resize();
                                                                                            });
                                                                                        })} style={format!("border-bottom: 1px solid #CCC;padding: 0.5em;{}", background_color)}>
                                                                                            { resource.name.clone() }
                                                                                        </div>
                                                                                        <Show condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;padding:0.25em;overflow: auto;">
                                                                                            { config_detail_view(&extension_configuration) }
                                                                                        </Show>
                                                                                    </div>
                                                                                }
                                                                            })
                                                                        }
                                                                    </div>
                                                                </div>
                                                            }
                                                        }
                                                    }
                                                </Show>
                                            </div>
                                        }
                                    })
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

async fn query_extension_list(
    extension_list: &UseStateHandle<Vec<Extension>>,
) -> Result<Vec<Extension>, LightString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
}

async fn read_environment_detail(
    detail: &UseStateHandle<Option<Environment>>,
    id: Id,
) -> Result<Environment, LightString> {
    let params = ReadEnvironmentReq { id: id };
    let environment = ReadEnvironmentApi.call(&params).await?;
    detail.set(Some(environment.clone()));
    return Ok(environment);
}

async fn read_environment_schema_detail(
    detail: &UseStateHandle<Option<EnvironmentSchema>>,
    environment_schema_id: Id,
) -> Result<EnvironmentSchema, LightString> {
    let params = ReadEnvironmentSchemaReq {
        id: environment_schema_id,
    };
    let environment_schema = ReadEnvironmentSchemaApi.call(&params).await?;
    detail.set(Some(environment_schema.clone()));
    return Ok(environment_schema);
}
