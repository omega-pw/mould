use super::super::extension::wrap_content;
use crate::components::button::Button;
use crate::components::r#if::If;
use crate::components::rich_text::render_rich_rext;
use crate::components::running::Running;
use crate::components::show::Show;
use crate::components::File;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::LightString;
use js_sys::JSON;
use sdk::job::continue_job::ContinueJobApi;
use sdk::job::continue_job::ContinueJobReq;
use sdk::job_record::enums::RecordStatus;
use sdk::job_record::enums::StepRecordStatus;
use sdk::job_record::enums::StepResourceRecordStatus;
use sdk::job_record::read_job_record::JobRecord;
use sdk::job_record::read_job_record::LogLevel;
use sdk::job_record::read_job_record::ReadJobRecordApi;
use sdk::job_record::read_job_record::ReadJobRecordReq;
use sdk::job_record::read_job_record::StepRecord;
use sdk::job_record::read_job_record::StepResLog;
use serde_json::Value;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tihu::datetime_format::FORMAT;
use tihu::Id;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Id,
}

#[function_component]
pub fn JobRecordDetail(props: &Props) -> Html {
    let is_saving: UseStateHandle<bool> = use_state(|| false);
    let active_job_step_record_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let active_step_resource_record_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let detail: UseStateHandle<Option<JobRecord>> = use_state(|| None);
    let id = props.id;
    let detail_clone = detail.clone();
    use_effect_with(id, move |_| {
        let destroyed = Arc::new(AtomicBool::new(false));
        let destroyed_clone = destroyed.clone();
        wasm_bindgen_futures::spawn_local(async move {
            start_read_loop(&detail_clone, id, &destroyed_clone).await;
        });
        move || {
            destroyed.store(true, Ordering::Relaxed);
        }
    });
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"任务名称："}</td>
                    <td>{detail.as_ref().map(|job_record|{job_record.job_name.as_ref().map(|job_name|html!{job_name}).unwrap_or_else(utils::empty_html)}).unwrap_or_else(utils::empty_html)}</td>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"执行环境："}</td>
                    <td>{detail.as_ref().map(|job_record|{job_record.environment_name.as_ref().map(|environment_name|html!{environment_name}).unwrap_or_else(utils::empty_html)}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"执行状态："}</td>
                    <td colspan="3">
                        {
                            detail.as_ref().map(|job_record|{
                                render_record_status(job_record.status)
                            }).unwrap_or_else(utils::empty_html)
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"创建时间："}</td>
                    <td>{detail.as_ref().map(|job_record|{html!{&job_record.created_time}}).unwrap_or_else(utils::empty_html)}</td>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"更新时间："}</td>
                    <td>{detail.as_ref().map(|job_record|{html!{&job_record.last_modified_time}}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:20em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"步骤列表:"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        {
                            detail.as_ref().map(|job_record|{
                                render_steps(&active_job_step_record_id, &active_step_resource_record_id, &job_record.step_record_list, &is_saving)
                            }).unwrap_or_else(utils::empty_html)
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

fn render_record_status(status: RecordStatus) -> Html {
    match status {
        RecordStatus::Running => html! {
            <span style="color:green;">
                {status.to_string()}
                <Running />
            </span>
        },
        RecordStatus::Success => html! {
            <span style="color:green;">{status.to_string()}</span>
        },
        RecordStatus::Failure => html! {
            <span style="color:red;">{status.to_string()}</span>
        },
    }
}

fn render_step_status(status: StepRecordStatus, show_running: bool) -> Html {
    match status {
        StepRecordStatus::Pending => html! {
            <span style="color:gray;">{status.to_string()}</span>
        },
        StepRecordStatus::Running => html! {
            <span style="color:green;">
                {status.to_string()}
                <If condition={show_running}>
                    <Running />
                </If>
            </span>
        },
        StepRecordStatus::Success => html! {
            <span style="color:green;">{status.to_string()}</span>
        },
        StepRecordStatus::Failure => html! {
            <span style="color:red;">{status.to_string()}</span>
        },
    }
}

fn render_step_resource_status(status: StepResourceRecordStatus) -> Html {
    match status {
        StepResourceRecordStatus::Pending => html! {
            <span style="color:gray;">{status.to_string()}</span>
        },
        StepResourceRecordStatus::Running => html! {
            <span style="color:green;">
                {status.to_string()}
                <Running />
            </span>
        },
        StepResourceRecordStatus::Success => html! {
            <span style="color:green;">{status.to_string()}</span>
        },
        StepResourceRecordStatus::Failure => html! {
            <span style="color:red;">{status.to_string()}</span>
        },
    }
}

fn render_steps(
    active_job_step_record_id: &UseStateHandle<Option<Id>>,
    active_step_resource_record_id: &UseStateHandle<Option<Id>>,
    steps: &[StepRecord],
    is_saving: &UseStateHandle<bool>,
) -> Html {
    html! {
        for steps.iter().map(|step_record| {
            let active_job_step_record_id = active_job_step_record_id.clone();
            let job_step_record_id = get_step_record_id(step_record);
            let is_active = active_job_step_record_id.deref() == &Some(job_step_record_id);
            let background_color = if is_active {
                "background-color: #EEE"
            } else {
                ""
            };
            html! {
                <div>
                    <div onclick={Callback::from(move |_| {
                        let active_job_step_record_id = active_job_step_record_id.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            active_job_step_record_id.set(Some(job_step_record_id));
                            utils::wait(0).await;
                            utils::trigger_resize();
                        });
                    })} style={format!("border-bottom: 1px solid #CCC;padding: 0.5em;display:flex;justify-content: space-between;{}", background_color)}>
                        {
                            format!("{}{}", get_step_name(step_record), match step_record {
                                StepRecord::Auto {..} => "",
                                StepRecord::Manual {..} => "(手动)",
                            })
                        }
                        {
                            match step_record {
                                StepRecord::Auto {job_step_record, ..} => render_step_status(job_step_record.status, true),
                                StepRecord::Manual {job_step_record, ..} => render_step_status(job_step_record.status, false),
                            }
                        }
                    </div>
                    <Show condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;display:flex;flex-direction:column;overflow: auto;">
                        <table style="width: 100%;table-layout: fixed;">
                            <tr>
                                <td class="align-right" style="vertical-align: top; width:6em;">{"步骤名称："}</td>
                                <td>
                                    { get_step_name(step_record) }
                                </td>
                            </tr>
                            <tr>
                                <td class="align-right" style="vertical-align: top;">{"备注："}</td>
                                <td>
                                    {
                                        if let Some(step_remark) = get_step_remark(step_record) {
                                            let content = render_rich_rext(&JSON::parse(step_remark).unwrap()).unwrap();
                                            wrap_content(content)
                                        } else {
                                            html!{}
                                        }
                                    }
                                </td>
                            </tr>
                            {
                                match step_record {
                                    StepRecord::Auto { .. } => {
                                        html! {}
                                    }
                                    StepRecord::Manual { job_step_record } => {
                                        let files: Vec<File> = job_step_record.attachments.as_ref().map(|attachments| {
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
                                                                    let url = format!("/blob/{}", key);
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
                                    },
                                }
                            }
                        </table>
                        {
                            match step_record {
                                StepRecord::Auto {
                                    step_resource_record_list,
                                    ..
                                } => {
                                    html! {
                                        <div style="border-top: 1px solid #CCC;flex-grow:1;position: relative;overflow: auto;">
                                            <div style="width:20em;height:100%;display:flex;flex-direction:column;padding: 0.5em 0;border-right: 1px solid #CCC;box-sizing: border-box;">
                                                <div style="border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源列表:"}</div>
                                                <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                                    {
                                                        for step_resource_record_list.iter().map(|step_resource_record| {
                                                            let step_resource_record_id = step_resource_record.id;
                                                            let active_step_resource_record_id = active_step_resource_record_id.clone();
                                                            let is_active = active_step_resource_record_id.deref() == &Some(step_resource_record_id);
                                                            let background_color = if is_active {
                                                                "background-color: #EEE"
                                                            } else {
                                                                ""
                                                            };
                                                            html! {
                                                                <div>
                                                                    <div onclick={Callback::from(move |_| {
                                                                        let active_step_resource_record_id = active_step_resource_record_id.clone();
                                                                        wasm_bindgen_futures::spawn_local(async move {
                                                                            active_step_resource_record_id.set(Some(step_resource_record_id));
                                                                            utils::wait(0).await;
                                                                            utils::trigger_resize();
                                                                        });
                                                                    })} style={format!("border-bottom: 1px solid #CCC;padding: 0.5em;display:flex;justify-content: space-between;{}", background_color)}>
                                                                        {step_resource_record.resource_name.clone()}
                                                                        {render_step_resource_status(step_resource_record.status)}
                                                                    </div>
                                                                    <Show condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;padding:0.25em;overflow: auto;">
                                                                        <div>{"输出:"}</div>
                                                                        {
                                                                            if let Some(output) = step_resource_record.output.as_ref() {
                                                                                html! {
                                                                                    <div>
                                                                                        {render_output(output)}
                                                                                    </div>
                                                                                }
                                                                            } else {
                                                                                html! {}
                                                                            }
                                                                        }
                                                                    </Show>
                                                                </div>
                                                            }
                                                        })
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }
                                StepRecord::Manual {
                                    job_step_record
                                } => {
                                    let record_id = job_step_record.record_id;
                                    let step_record_id = job_step_record.id;
                                    let is_saving_clone = is_saving.clone();
                                    let on_continue = Callback::from(move |_| {
                                        let is_saving = is_saving_clone.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            continue_job(record_id, step_record_id, true, is_saving).await.ok();
                                        });
                                    });
                                    let is_saving_clone = is_saving.clone();
                                    let on_stop = Callback::from(move |_| {
                                        let is_saving = is_saving_clone.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            continue_job(record_id, step_record_id, false, is_saving).await.ok();
                                        });
                                    });
                                    html! {
                                        <div style="border: 1px solid #CCC;padding: 0.5em;margin-bottom: 0.5em;">
                                            <If condition={StepRecordStatus::Running==job_step_record.status}>
                                                <Button onclick={on_continue} style="margin-left:0.5em;">{"继续"}</Button>
                                                <Button onclick={on_stop} style="margin-left:0.5em;">{"终止"}</Button>
                                            </If>
                                        </div>
                                    }
                                },
                            }
                        }
                    </Show>
                </div>
            }
        })
    }
}

fn get_step_record_id(step_record: &StepRecord) -> Id {
    match step_record {
        StepRecord::Auto {
            job_step_record, ..
        } => job_step_record.id,
        StepRecord::Manual {
            job_step_record, ..
        } => job_step_record.id,
    }
}

fn get_step_name(step_record: &StepRecord) -> &String {
    match step_record {
        StepRecord::Auto {
            job_step_record, ..
        } => &job_step_record.step_name,
        StepRecord::Manual {
            job_step_record, ..
        } => &job_step_record.step_name,
    }
}

fn get_step_remark(step_record: &StepRecord) -> &Option<String> {
    match step_record {
        StepRecord::Auto {
            job_step_record, ..
        } => &job_step_record.step_remark,
        StepRecord::Manual {
            job_step_record, ..
        } => &job_step_record.step_remark,
    }
}

fn render_output(output: &str) -> Html {
    let logs: Vec<StepResLog> = serde_json::from_str(output).unwrap_or_default();
    let mut list = Vec::new();
    for log in logs {
        let color = match log.level {
            LogLevel::Error => Some("red"),
            _ => None,
        };
        let style = color.map(|color| format!("color: {}", color));
        let mut sub_list = Vec::new();
        for (index, item) in log
            .content
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .split("\n")
            .enumerate()
        {
            if 0 < index {
                sub_list.push(html! {
                    <br/>
                });
            }
            if !item.is_empty() {
                sub_list.push(Html::from(item));
            }
        }
        list.push(html! {
            <p style={style.clone()}>
                { format!("{} {} ", log.time.format(FORMAT), log.level) }
                { for sub_list.into_iter() }
            </p>
        });
    }
    html! { for list.into_iter() }
}

async fn read_job_record_detail(
    detail: &UseStateHandle<Option<JobRecord>>,
    id: Id,
    first_time: bool,
) -> Result<JobRecord, LightString> {
    let params = ReadJobRecordReq { id: id };
    let job_record = if first_time {
        ReadJobRecordApi.call(&params).await?
    } else {
        ReadJobRecordApi
            .disable_loading()
            .unwrap_error_handler(|err| -> Pin<Box<dyn Future<Output = ()>>> {
                log::error!("{}", err);
                Box::pin(async {})
            })
            .req_error_handler(|err| -> Pin<Box<dyn Future<Output = ()>>> {
                log::error!("{}", err);
                Box::pin(async {})
            })
            .call(&params)
            .await?
    };
    detail.set(Some(job_record.clone()));
    return Ok(job_record);
}

async fn start_read_loop(
    detail: &UseStateHandle<Option<JobRecord>>,
    id: Id,
    destroyed: &AtomicBool,
) {
    let mut times: usize = 0;
    loop {
        if destroyed.load(Ordering::Relaxed) {
            break;
        } else {
            match read_job_record_detail(detail, id, 0 == times).await {
                Ok(detail) => {
                    if RecordStatus::Running == detail.status {
                        times += 1;
                        utils::wait(2000).await;
                    } else {
                        break;
                    }
                }
                Err(_err) => {
                    times += 1;
                    utils::wait(2000).await;
                }
            }
        }
    }
}

async fn continue_job(
    record_id: Id,
    step_record_id: Id,
    success: bool,
    is_saving: UseStateHandle<bool>,
) -> Result<(), LightString> {
    ContinueJobApi
        .lock_handler(is_saving)
        .call(&ContinueJobReq {
            record_id: record_id,
            step_record_id: step_record_id,
            success: success,
        })
        .await?;
    utils::success(LightString::from("操作成功"));
    return Ok(());
}
