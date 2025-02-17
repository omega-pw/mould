use super::super::extension::config_detail_view;
use super::super::extension::get_parameter_schema;
use super::super::extension::parse_config;
use super::super::extension::wrap_content;
use crate::components::rich_text::render_rich_rext;
use crate::components::show::Show;
use crate::components::File;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::LightString;
use js_sys::JSON;
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
use std::ops::Deref;
use tihu::Id;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Id,
}

#[function_component]
pub fn JobDetail(props: &Props) -> Html {
    let active_job_step_id: UseStateHandle<Option<Id>> = use_state(|| Default::default());
    let extension_list: UseStateHandle<Vec<Extension>> = use_state(|| Default::default());
    let environment_schema_detail: UseStateHandle<Option<EnvironmentSchema>> = use_state(|| None);
    let detail: UseStateHandle<Option<Job>> = use_state(|| None);
    let id = props.id;
    let extension_list_clone = extension_list.clone();
    let detail_clone = detail.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    use_effect_with(id, move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            query_extension_list(&extension_list_clone).await.ok();
            read_job_detail(&detail_clone, &environment_schema_detail_clone, id)
                .await
                .ok();
        });
        || ()
    });
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"任务名称："}</td>
                    <td>{detail.as_ref().map(|job|{html!{&job.name}}).unwrap_or_else(utils::empty_html)}</td>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"环境规格："}</td>
                    <td>{environment_schema_detail.as_ref().map(|environment_schema|{html!{&environment_schema.name}}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"备注："}</td>
                    <td colspan="3">{detail.as_ref().map(|job|{job.remark.as_ref().map(|remark|{html!{remark}}).unwrap_or_else(utils::empty_html)}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:24em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"步骤列表"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        {
                            if let Some(detail) = detail.as_ref() {
                                html! {
                                    for detail.job_step_list.iter().map(|job_step| {
                                        let active_job_step_id = active_job_step_id.clone();
                                        let job_step_id = get_step_id(job_step);
                                        let is_active = active_job_step_id.deref() == &Some(job_step_id);
                                        let background_color = if is_active {
                                            "background-color: #EEE"
                                        } else {
                                            ""
                                        };
                                        html! {
                                            <div>
                                                <div onclick={Callback::from(move |_| {
                                                    let active_job_step_id = active_job_step_id.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        active_job_step_id.set(Some(job_step_id));
                                                        utils::wait(0).await;
                                                        utils::trigger_resize();
                                                    });
                                                })} style={format!("border-bottom: 1px solid #CCC;padding: 0.5em;{}", background_color)}>
                                                    { get_step_name(job_step) }
                                                </div>
                                                <Show condition={is_active} style="position:absolute;left:24em;right:0;top:0;bottom:0;overflow: auto;">
                                                    <table style="width: 100%;table-layout: fixed;">
                                                        <tr>
                                                            <td class="align-right" style="vertical-align: top; width:6em;">{"步骤名称："}</td>
                                                            <td>
                                                                { get_step_name(job_step) }
                                                            </td>
                                                        </tr>
                                                        <tr>
                                                            <td class="align-right" style="vertical-align: top;">{"备注："}</td>
                                                            <td>
                                                                {
                                                                    if let Some(step_remark) = get_step_remark(job_step) {
                                                                        let content = render_rich_rext(&JSON::parse(step_remark).unwrap()).unwrap();
                                                                        wrap_content(content)
                                                                    } else {
                                                                        html!{}
                                                                    }
                                                                }
                                                            </td>
                                                        </tr>
                                                        {
                                                            match job_step {
                                                                JobStep::Auto { schema_resource_id, operation_id, operation_name, operation_parameter, .. } => {
                                                                    let schema_resource = environment_schema_detail.as_ref().map(|environment_schema_detail|environment_schema_detail.resource_list.iter().find(|resource| &resource.id == schema_resource_id)).flatten();
                                                                    if let Some(schema_resource) = schema_resource {
                                                                        let operation_parameter = extension_list
                                                                        .iter()
                                                                        .find(|extension| extension.id == schema_resource.extension_id)
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
                                                                        .unwrap_or_default();
                                                                        html! {
                                                                            <>
                                                                                <tr>
                                                                                    <td class="align-right" style="vertical-align: top;">{"操作资源："}</td>
                                                                                    <td>{ schema_resource.name.clone() }</td>
                                                                                </tr>
                                                                                <tr>
                                                                                    <td class="align-right" style="vertical-align: top;">{"操作类型："}</td>
                                                                                    <td>{ operation_name }</td>
                                                                                </tr>
                                                                                <tr>
                                                                                    <td class="align-right" style="vertical-align: top;">{"操作参数："}</td>
                                                                                    <td>
                                                                                        {
                                                                                            config_detail_view(&operation_parameter)
                                                                                        }
                                                                                    </td>
                                                                                </tr>
                                                                            </>
                                                                        }
                                                                    } else {
                                                                        html! {
                                                                            <tr>
                                                                                <td class="align-right" style="vertical-align: top;">{"操作资源"}</td>
                                                                                <td>{"资源规格已被移除"}</td>
                                                                            </tr>
                                                                        }
                                                                    }
                                                                },
                                                                JobStep::Manual { attachments, .. } => {
                                                                    let files: Vec<File> = attachments.as_ref().map(|attachments| {
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
                                                                                                File::Remote {
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
                                                                    html! {
                                                                        <tr>
                                                                            <td class="align-right" style="vertical-align: top;">{"附件："}</td>
                                                                            <td>
                                                                                {
                                                                                    for files.iter().map(|file| {
                                                                                        match file {
                                                                                            File::Remote { key, name, .. } => {
                                                                                                let url = format!("/{}", key);
                                                                                                html! {
                                                                                                    <div>
                                                                                                        <a href={url} target="_blank" download={name.clone()}>{name}</a>
                                                                                                    </div>
                                                                                                }
                                                                                            }
                                                                                            File::Local(hashing_file) => {
                                                                                                html! { hashing_file.file.name() }
                                                                                            }
                                                                                        }
                                                                                    })
                                                                                }
                                                                            </td>
                                                                        </tr>
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    </table>
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

async fn read_job_detail(
    detail: &UseStateHandle<Option<Job>>,
    environment_schema_detail: &UseStateHandle<Option<EnvironmentSchema>>,
    id: Id,
) -> Result<(), LightString> {
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
    extension_list: &UseStateHandle<Vec<Extension>>,
) -> Result<Vec<Extension>, LightString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
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
