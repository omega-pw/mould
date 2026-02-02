use super::super::extension::get_parameter_schema;
use super::super::extension::parse_config;
use super::super::extension::wrap_content;
use super::super::extension::ConfigDetailView;
use crate::components::rich_text::render_rich_rext;
use crate::components::visable::Visable;
use crate::components::ResourceMetadata;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::extension::query_extension::QueryExtensionApi;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::Extension;
use sdk::job::read_job::Job;
use sdk::job::read_job::JobStep;
use sdk::job::read_job::ReadJobApi;
use sdk::job::read_job::ReadJobReq;
use serde_json::Value;
use tihu::Id;

#[component]
pub fn JobDetail(#[prop(optional)] id: Id) -> impl IntoView {
    let active_job_step_id: RwSignal<Option<Id>> = RwSignal::new(Default::default());
    let extension_list: RwSignal<Vec<Extension>> = RwSignal::new(Default::default());
    let environment_schema_detail: RwSignal<Option<EnvironmentSchema>> = RwSignal::new(None);
    let detail: RwSignal<Option<Job>> = RwSignal::new(None);
    let extension_list_clone = extension_list.clone();
    let detail_clone = detail.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    wasm_bindgen_futures::spawn_local(async move {
        query_extension_list(&extension_list_clone).await.ok();
        read_job_detail(&detail_clone, &environment_schema_detail_clone, id)
            .await
            .ok();
    });
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"任务名称："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|job| job.name.clone())
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
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"备注："}</td>
                    <td colspan="3">
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|job| job.remark.clone()).flatten()
                        }
                    </td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:24em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"步骤列表"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        {
                            let detail = detail.clone();
                            move || {
                                if let Some(detail) = detail.get() {
                                    view! {
                                        <For
                                            each={
                                                let job_step_list = detail.job_step_list.clone();
                                                move || { job_step_list.clone().into_iter() }
                                            }
                                            key=|job_step| { get_step_id(job_step) }
                                            children=move |job_step| {
                                                let active_job_step_id = active_job_step_id.clone();
                                                let job_step_id = get_step_id(&job_step);
                                                let is_active = {
                                                    let active_job_step_id = active_job_step_id.clone();
                                                    let job_step_id = job_step_id.clone();
                                                    move || {
                                                        &active_job_step_id.read() == &Some(job_step_id.clone())
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
                                                            let active_job_step_id = active_job_step_id.clone();
                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                active_job_step_id.set(Some(job_step_id));
                                                                utils::wait(0).await;
                                                                utils::trigger_resize();
                                                            });
                                                        }} style={move || format!("border-bottom: 1px solid #CCC;padding: 0.5em;{}", background_color())}>
                                                            { get_step_name(&job_step).clone() }
                                                        </div>
                                                        <Visable condition={is_active} style="position:absolute;left:24em;right:0;top:0;bottom:0;overflow: auto;">
                                                            <table style="width: 100%;table-layout: fixed;">
                                                                <tr>
                                                                    <td class="align-right" style="vertical-align: top; width:6em;">{"步骤名称："}</td>
                                                                    <td>
                                                                        { get_step_name(&job_step).clone() }
                                                                    </td>
                                                                </tr>
                                                                <tr>
                                                                    <td class="align-right" style="vertical-align: top;">{"备注："}</td>
                                                                    <td>
                                                                        {
                                                                            if let Some(step_remark) = get_step_remark(&job_step) {
                                                                                let content = render_rich_rext(step_remark).unwrap();
                                                                                wrap_content(content).into_any()
                                                                            } else {
                                                                                view!{}.into_any()
                                                                            }
                                                                        }
                                                                    </td>
                                                                </tr>
                                                                {
                                                                    match job_step {
                                                                        JobStep::Auto { schema_resource_id, operation_id, operation_name, operation_parameter, .. } => {
                                                                            (move || {
                                                                                let schema_resource = environment_schema_detail.read().as_ref().map(|environment_schema_detail|environment_schema_detail.resource_list.iter().find(|resource| resource.id == schema_resource_id).map(Clone::clone)).flatten();
                                                                                if let Some(schema_resource) = schema_resource {
                                                                                    let operation_parameter = {
                                                                                        let extension_id = schema_resource.extension_id.clone();
                                                                                        let operation_id = operation_id.clone();
                                                                                        let operation_parameter = operation_parameter.clone();
                                                                                        Signal::derive(move || {
                                                                                            extension_list
                                                                                            .read()
                                                                                            .iter()
                                                                                            .find(|extension| extension.id == extension_id)
                                                                                            .map(|extension| {
                                                                                                get_parameter_schema(
                                                                                                    &extension.operations,
                                                                                                    &operation_id,
                                                                                                )
                                                                                                .map(|parameter_schema| {
                                                                                                    parse_config(
                                                                                                        parameter_schema.clone(),
                                                                                                        &operation_parameter,
                                                                                                    )
                                                                                                })
                                                                                            }).flatten()
                                                                                            .unwrap_or_default()
                                                                                        })
                                                                                    };
                                                                                    view! {
                                                                                        <>
                                                                                            <tr>
                                                                                                <td class="align-right" style="vertical-align: top;">{"操作资源："}</td>
                                                                                                <td>{ schema_resource.name.clone() }</td>
                                                                                            </tr>
                                                                                            <tr>
                                                                                                <td class="align-right" style="vertical-align: top;">{"操作类型："}</td>
                                                                                                <td>{ operation_name.clone() }</td>
                                                                                            </tr>
                                                                                            <tr>
                                                                                                <td class="align-right" style="vertical-align: top;">{"操作参数："}</td>
                                                                                                <td>
                                                                                                    <ConfigDetailView attributes={operation_parameter}/>
                                                                                                </td>
                                                                                            </tr>
                                                                                        </>
                                                                                    }.into_any()
                                                                                } else {
                                                                                    view! {
                                                                                        <tr>
                                                                                            <td class="align-right" style="vertical-align: top;">{"操作资源"}</td>
                                                                                            <td>{"资源规格已被移除"}</td>
                                                                                        </tr>
                                                                                    }.into_any()
                                                                                }
                                                                            }).into_any()
                                                                        },
                                                                        JobStep::Manual { attachments, .. } => {
                                                                            let files: Vec<ResourceMetadata> = attachments.as_ref().map(|attachments| {
                                                                                serde_json::from_str::<Value>(&attachments).map(|value| {
                                                                                    value
                                                                                    .as_array()
                                                                                    .map(|value| {
                                                                                        value
                                                                                            .iter()
                                                                                            .map(|value| {
                                                                                                value
                                                                                                    .as_object()
                                                                                                    .map(|map| {
                                                                                                        let key = map
                                                                                                            .get("key")
                                                                                                            .unwrap()
                                                                                                            .as_str()
                                                                                                            .map(|value| value.to_string())
                                                                                                            .unwrap_or_default();
                                                                                                        let name = map
                                                                                                            .get("name")
                                                                                                            .unwrap()
                                                                                                            .as_str()
                                                                                                            .map(|value| value.to_string())
                                                                                                            .unwrap_or_default();
                                                                                                        let size = map.get("size").unwrap().as_f64().unwrap();
                                                                                                        let mime_type = map
                                                                                                            .get("mime_type")
                                                                                                            .unwrap()
                                                                                                            .as_str()
                                                                                                            .map(|value| value.to_string())
                                                                                                            .unwrap_or_default();
                                                                                                        ResourceMetadata {
                                                                                                            key: key,
                                                                                                            name: name,
                                                                                                            size: size,
                                                                                                            mime_type: mime_type,
                                                                                                        }
                                                                                                    })
                                                                                                    .unwrap()
                                                                                            })
                                                                                            .collect()
                                                                                    })
                                                                                    .unwrap_or_default()
                                                                                })
                                                                                .unwrap_or_default()
                                                                            })
                                                                            .unwrap_or_default();
                                                                            view! {
                                                                                <tr>
                                                                                    <td class="align-right" style="vertical-align: top;">{"附件："}</td>
                                                                                    <td>
                                                                                        <For
                                                                                            each={
                                                                                                let files = files.clone();
                                                                                                move || { files.clone().into_iter().enumerate() }
                                                                                            }
                                                                                            key=|(index, _file)| { *index }
                                                                                            children=move |(_index, metadata)| {
                                                                                                let url = format!("/{}", metadata.key);
                                                                                                view! {
                                                                                                    <div>
                                                                                                        <a href={url} target="_blank" download={metadata.name.clone()}>{metadata.name.clone()}</a>
                                                                                                    </div>
                                                                                                }.into_any()
                                                                                            }
                                                                                        />
                                                                                    </td>
                                                                                </tr>
                                                                            }.into_any()
                                                                        }
                                                                    }
                                                                }
                                                            </table>
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

async fn read_job_detail(
    detail: &RwSignal<Option<Job>>,
    environment_schema_detail: &RwSignal<Option<EnvironmentSchema>>,
    id: Id,
) -> Result<(), SharedString> {
    let params = ReadJobReq { id: id };
    let mut job = ReadJobApi.call(&params).await?;
    job.job_step_list.sort_by_key(|job_step| match job_step {
        sdk::job::read_job::JobStep::Auto { seq, .. } => *seq,
        sdk::job::read_job::JobStep::Manual { seq, .. } => *seq,
    });
    read_environment_schema_detail(environment_schema_detail, job.environment_schema_id).await?;
    detail.set(Some(job));
    return Ok(());
}

async fn query_extension_list(
    extension_list: &RwSignal<Vec<Extension>>,
) -> Result<Vec<Extension>, SharedString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
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

fn get_step_id(job_step: &JobStep) -> Id {
    match job_step {
        JobStep::Auto { id, .. } => *id,
        JobStep::Manual { id, .. } => *id,
    }
}

fn get_step_name(job_step: &JobStep) -> &String {
    match job_step {
        JobStep::Auto { name, .. } => name,
        JobStep::Manual { name, .. } => name,
    }
}

fn get_step_remark(job_step: &JobStep) -> Option<&String> {
    match job_step {
        JobStep::Auto { remark, .. } => remark.as_ref(),
        JobStep::Manual { remark, .. } => remark.as_ref(),
    }
}
