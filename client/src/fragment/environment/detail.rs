use super::super::extension::get_configuration_schema;
use super::super::extension::parse_config;
use super::super::extension::ConfigDetailView;
use crate::components::visable::Visable;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
use sdk::environment::read_environment::Environment;
use sdk::environment::read_environment::ReadEnvironmentApi;
use sdk::environment::read_environment::ReadEnvironmentReq;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::extension::query_extension::QueryExtensionApi;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::Extension;
use tihu::Id;

#[component]
pub fn EnvironmentDetail(#[prop(optional)] id: Id) -> impl IntoView {
    let active_schema_resource_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let active_resource_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let detail: RwSignal<Option<Environment>> = RwSignal::new(None);
    let environment_schema_detail: RwSignal<Option<EnvironmentSchema>> = RwSignal::new(None);
    let extension_list: RwSignal<Vec<Extension>> = RwSignal::new(Default::default());
    let extension_list_clone = extension_list.clone();
    let detail_clone = detail.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match query_extension_list(&extension_list_clone).await {
            Ok(_extension_list) => {
                if let Some(environment) = read_environment_detail(&detail_clone, id).await.ok() {
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
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"环境名称："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|environment| environment.name.clone())
                        }
                    </td>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"环境规格："}</td>
                    <td>
                        {
                            let environment_schema_detail = environment_schema_detail.clone();
                            move || environment_schema_detail.read().as_ref().map(|environment_schema| environment_schema.name.clone())
                        }
                    </td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:16em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源规格"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        {
                            let detail = detail.clone();
                            move || {
                                if let Some(detail) = detail.get() {
                                    view! {
                                        <For
                                            each={
                                                let schema_resource_list = detail.schema_resource_list.clone();
                                                move || { schema_resource_list.clone().into_iter() }
                                            }
                                            key=|schema_resource| { schema_resource.id.clone() }
                                            children=move |schema_resource| {
                                                let schema_resource_name = schema_resource.name.clone();
                                                let schema_resource_id = schema_resource.id;
                                                let extension_id = schema_resource.extension_id.clone();
                                                let extension_list = extension_list.clone();
                                                let is_active = {
                                                    let active_schema_resource_id = active_schema_resource_id.clone();
                                                    move || {
                                                        &active_schema_resource_id.read() == &Some(schema_resource_id.clone())
                                                    }
                                                };
                                                let background_color = {
                                                    let is_active = is_active.clone();
                                                    move || {
                                                        if is_active() {
                                                            "background-color: #EEE"
                                                        } else {
                                                            ""
                                                        }
                                                    }
                                                };
                                                view! {
                                                    <div>
                                                        <div on:click={move |_| {
                                                            let active_schema_resource_id = active_schema_resource_id.clone();
                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                active_schema_resource_id.set(Some(schema_resource_id));
                                                                utils::wait(0).await;
                                                                utils::trigger_resize();
                                                            });
                                                        }} style={move || format!("border-bottom: 1px solid #CCC;padding: 0.5em;{}", background_color())}>
                                                            {schema_resource_name}
                                                        </div>
                                                        <Visable condition={is_active} style="position:absolute;left:16em;right:0;top:0;bottom:0;overflow: auto;">
                                                            {
                                                                let active_resource_id = active_resource_id.clone();
                                                                view! {
                                                                    <div style="width:20em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                                                                        <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源列表"}</div>
                                                                        <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                                                            <For
                                                                                each={
                                                                                    let resource_list = schema_resource.resource_list.clone();
                                                                                    move || { resource_list.clone().into_iter() }
                                                                                }
                                                                                key=|resource| { resource.id.clone() }
                                                                                children=move |resource| {
                                                                                    let resource_id = resource.id;
                                                                                    let is_active = {
                                                                                        let active_resource_id = active_resource_id.clone();
                                                                                        let resource_id = resource_id.clone();
                                                                                        move || {
                                                                                            &active_resource_id.read() == &Some(resource_id.clone())
                                                                                        }
                                                                                    };
                                                                                    let background_color = {
                                                                                        let is_active = is_active.clone();
                                                                                        move || {
                                                                                            if is_active() {
                                                                                                "background-color: #EEE"
                                                                                            } else {
                                                                                                ""
                                                                                            }
                                                                                        }
                                                                                    };
                                                                                    let extension_configuration = {
                                                                                        let extension_id = extension_id.clone();
                                                                                        Signal::derive(move || {
                                                                                            let configuration_schema =
                                                                                            get_configuration_schema(&extension_list.read(), &extension_id)
                                                                                                .map(|configuration_schema| configuration_schema.clone())
                                                                                                .unwrap_or_default();
                                                                                            parse_config(
                                                                                                configuration_schema.clone(),
                                                                                                &resource.extension_configuration,
                                                                                            )
                                                                                        })
                                                                                    };
                                                                                    view! {
                                                                                        <div>
                                                                                            <div on:click={move |_| {
                                                                                                let active_resource_id = active_resource_id.clone();
                                                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                                                    active_resource_id.set(Some(resource_id));
                                                                                                    utils::wait(0).await;
                                                                                                    utils::trigger_resize();
                                                                                                });
                                                                                            }} style={move || format!("border-bottom: 1px solid #CCC;padding: 0.5em;{}", background_color())}>
                                                                                                { resource.name.clone() }
                                                                                            </div>
                                                                                            <Visable condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;padding:0.25em;overflow: auto;">
                                                                                                <ConfigDetailView attributes={extension_configuration}/>
                                                                                            </Visable>
                                                                                        </div>
                                                                                    }
                                                                                }
                                                                            />
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }
                                                        </Visable>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }
                            }
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

async fn query_extension_list(
    extension_list: &RwSignal<Vec<Extension>>,
) -> Result<Vec<Extension>, SharedString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
}

async fn read_environment_detail(
    detail: &RwSignal<Option<Environment>>,
    id: Id,
) -> Result<Environment, SharedString> {
    let params = ReadEnvironmentReq { id: id };
    let environment = ReadEnvironmentApi.call(&params).await?;
    detail.set(Some(environment.clone()));
    return Ok(environment);
}

async fn read_environment_schema_detail(
    detail: &RwSignal<Option<EnvironmentSchema>>,
    environment_schema_id: Id,
) -> Result<EnvironmentSchema, SharedString> {
    let params = ReadEnvironmentSchemaReq {
        id: environment_schema_id,
    };
    let environment_schema = ReadEnvironmentSchemaApi.call(&params).await?;
    detail.set(Some(environment_schema.clone()));
    return Ok(environment_schema);
}
