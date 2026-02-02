use super::super::extension::wrap_content;
use crate::components::button::Button;
use crate::components::rich_text::render_rich_rext;
use crate::components::running::Running;
use crate::components::visable::Visable;
use crate::components::ResourceMetadata;
use crate::sdk;
use crate::utils;
use crate::utils::format_time_local;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
use leptos::Params;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
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
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tihu::datetime_format::FORMAT;
use tihu::Id;

#[derive(Params, Clone, PartialEq)]
struct JobRecordDetailParams {
    id: Option<Id>,
}

#[component]
pub fn JobRecordDetailPage() -> impl IntoView {
    let detail_params = use_params::<JobRecordDetailParams>();
    move || {
        let detail_params = detail_params.get();
        match detail_params {
            Ok(detail_params) => {
                if let Some(id) = detail_params.id {
                    view! {
                        <JobRecordDetail id={id}/>
                    }
                    .into_any()
                } else {
                    view! {
                        <div>
                            {"参数错误: id为空"}
                        </div>
                    }
                    .into_any()
                }
            }
            Err(err) => view! {
                <div>
                    {format!("参数错误: {}", err)}
                </div>
            }
            .into_any(),
        }
    }
}

#[component]
pub fn JobRecordDetail(#[prop(optional)] id: Id) -> impl IntoView {
    let is_saving: RwSignal<bool> = RwSignal::new(false);
    let active_job_step_record_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let active_step_resource_record_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let detail: RwSignal<Option<JobRecord>> = RwSignal::new(None);
    let detail_clone = detail.clone();
    let destroyed = Arc::new(AtomicBool::new(false));
    let destroyed_clone = destroyed.clone();
    wasm_bindgen_futures::spawn_local(async move {
        start_read_loop(&detail_clone, id, &destroyed_clone).await;
    });
    on_cleanup(move || {
        destroyed.store(true, Ordering::Relaxed);
    });
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"任务名称："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|job_record| job_record.job_name.clone()).flatten()
                        }
                    </td>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"执行环境："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|job_record| job_record.environment_name.clone()).flatten()
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"执行状态："}</td>
                    <td colspan="3">
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|job_record| render_record_status(job_record.status))
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"创建时间："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|job_record| format_time_local(&job_record.created_time).to_string())
                        }
                    </td>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"更新时间："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|job_record| format_time_local(&job_record.last_modified_time).to_string())
                        }
                    </td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:20em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"步骤列表:"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        {
                            let detail = detail.clone();
                            move || {
                                if let Some(job_record) = detail.get() {
                                    render_steps(active_job_step_record_id.clone(), active_step_resource_record_id.clone(), &job_record.step_record_list, is_saving.clone()).into_any()
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

fn render_record_status(status: RecordStatus) -> impl IntoView {
    match status {
        RecordStatus::Running => view! {
            <span style="color:green;">
                {status.to_string()}
                <Running />
            </span>
        }
        .into_any(),
        RecordStatus::Success => view! {
            <span style="color:green;">{status.to_string()}</span>
        }
        .into_any(),
        RecordStatus::Failure => view! {
            <span style="color:red;">{status.to_string()}</span>
        }
        .into_any(),
    }
}

fn render_step_status(status: StepRecordStatus, show_running: bool) -> impl IntoView {
    match status {
        StepRecordStatus::Pending => view! {
            <span style="color:gray;">{status.to_string()}</span>
        }
        .into_any(),
        StepRecordStatus::Running => view! {
            <span style="color:green;">
                {status.to_string()}
                <Show when={move || show_running}>
                    <Running />
                </Show>
            </span>
        }
        .into_any(),
        StepRecordStatus::Success => view! {
            <span style="color:green;">{status.to_string()}</span>
        }
        .into_any(),
        StepRecordStatus::Failure => view! {
            <span style="color:red;">{status.to_string()}</span>
        }
        .into_any(),
    }
}

fn render_step_resource_status(status: StepResourceRecordStatus) -> impl IntoView {
    match status {
        StepResourceRecordStatus::Pending => view! {
            <span style="color:gray;">{status.to_string()}</span>
        }
        .into_any(),
        StepResourceRecordStatus::Running => view! {
            <span style="color:green;">
                {status.to_string()}
                <Running />
            </span>
        }
        .into_any(),
        StepResourceRecordStatus::Success => view! {
            <span style="color:green;">{status.to_string()}</span>
        }
        .into_any(),
        StepResourceRecordStatus::Failure => view! {
            <span style="color:red;">{status.to_string()}</span>
        }
        .into_any(),
    }
}

fn render_steps(
    active_job_step_record_id: RwSignal<Option<Id>>,
    active_step_resource_record_id: RwSignal<Option<Id>>,
    steps: &Vec<StepRecord>,
    is_saving: RwSignal<bool>,
) -> impl IntoView + use<> {
    view! {
        <For
            each={
                let steps = steps.clone();
                move || { steps.clone().into_iter() }
            }
            key=|step_record| { get_step_record_id(step_record) }
            children=move |step_record| {
                let job_step_record_id = get_step_record_id(&step_record);
                let is_active = {
                    let active_job_step_record_id = active_job_step_record_id.clone();
                    let job_step_record_id = job_step_record_id.clone();
                    move || {
                        &active_job_step_record_id.read() == &Some(job_step_record_id.clone())
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
                            let active_job_step_record_id = active_job_step_record_id.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                active_job_step_record_id.set(Some(job_step_record_id));
                                utils::wait(0).await;
                                utils::trigger_resize();
                            });
                        }} style={move || format!("border-bottom: 1px solid #CCC;padding: 0.5em;display:flex;justify-content: space-between;{}", background_color())}>
                            {
                                format!("{}{}", get_step_name(&step_record), match &step_record {
                                    StepRecord::Auto {..} => "",
                                    StepRecord::Manual {..} => "(手动)",
                                })
                            }
                            {
                                match &step_record {
                                    StepRecord::Auto {job_step_record, ..} => render_step_status(job_step_record.status, true),
                                    StepRecord::Manual {job_step_record, ..} => render_step_status(job_step_record.status, false),
                                }
                            }
                        </div>
                        <Visable condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;display:flex;flex-direction:column;overflow: auto;">
                            <table style="width: 100%;table-layout: fixed;">
                                <tr>
                                    <td class="align-right" style="vertical-align: top; width:6em;">{"步骤名称："}</td>
                                    <td>
                                        { get_step_name(&step_record).clone() }
                                    </td>
                                </tr>
                                <tr>
                                    <td class="align-right" style="vertical-align: top;">{"备注："}</td>
                                    <td>
                                        {
                                            if let Some(step_remark) = get_step_remark(&step_record) {
                                                let content = render_rich_rext(step_remark).unwrap();
                                                wrap_content(content).into_any()
                                            } else {
                                                view!{}.into_any()
                                            }
                                        }
                                    </td>
                                </tr>
                                {
                                    match &step_record {
                                        StepRecord::Auto { .. } => {
                                            view! {}.into_any()
                                        }
                                        StepRecord::Manual { job_step_record } => {
                                            let files: Vec<ResourceMetadata> = job_step_record.attachments.as_ref().map(|attachments| {
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
                                                            children=move |(_, metadata)| {
                                                                let url = format!("/{}", metadata.key);
                                                                view! {
                                                                    <div>
                                                                        <a href={url} target="_blank" download={metadata.name.clone()}>{metadata.name.clone()}</a>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </td>
                                                </tr>
                                            }.into_any()
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
                                        view! {
                                            <div style="border-top: 1px solid #CCC;flex-grow:1;position: relative;overflow: auto;">
                                                <div style="width:20em;height:100%;display:flex;flex-direction:column;padding: 0.5em 0;border-right: 1px solid #CCC;box-sizing: border-box;">
                                                    <div style="border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源列表:"}</div>
                                                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                                        <For
                                                            each={
                                                                let step_resource_record_list = step_resource_record_list.clone();
                                                                move || { step_resource_record_list.clone().into_iter() }
                                                            }
                                                            key=|step_resource_record| { step_resource_record.id.clone() }
                                                            children=move |step_resource_record| {
                                                                let step_resource_record_id = step_resource_record.id;
                                                                let is_active = {
                                                                    let active_step_resource_record_id = active_step_resource_record_id.clone();
                                                                    let step_resource_record_id = step_resource_record_id.clone();
                                                                    move || {
                                                                        &active_step_resource_record_id.read() == &Some(step_resource_record_id.clone())
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
                                                                            let active_step_resource_record_id = active_step_resource_record_id.clone();
                                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                                active_step_resource_record_id.set(Some(step_resource_record_id));
                                                                                utils::wait(0).await;
                                                                                utils::trigger_resize();
                                                                            });
                                                                        }} style={move || format!("border-bottom: 1px solid #CCC;padding: 0.5em;display:flex;justify-content: space-between;{}", background_color())}>
                                                                            {step_resource_record.resource_name.clone()}
                                                                            {render_step_resource_status(step_resource_record.status)}
                                                                        </div>
                                                                        <Visable condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;padding:0.25em;overflow: auto;">
                                                                            <div>{"输出:"}</div>
                                                                            {
                                                                                if let Some(output) = step_resource_record.output.as_ref() {
                                                                                    view! {
                                                                                        <div>
                                                                                            {render_output(output)}
                                                                                        </div>
                                                                                    }.into_any()
                                                                                } else {
                                                                                    view! {}.into_any()
                                                                                }
                                                                            }
                                                                        </Visable>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                    StepRecord::Manual {
                                        job_step_record
                                    } => {
                                        let record_id = job_step_record.record_id;
                                        let step_record_id = job_step_record.id;
                                        let is_saving_clone = is_saving.clone();
                                        let on_continue = UnsyncCallback::new(move |_| {
                                            let is_saving = is_saving_clone.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                continue_job(record_id, step_record_id, true, is_saving).await.ok();
                                            });
                                        });
                                        let is_saving_clone = is_saving.clone();
                                        let on_stop = UnsyncCallback::new(move |_| {
                                            let is_saving = is_saving_clone.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                continue_job(record_id, step_record_id, false, is_saving).await.ok();
                                            });
                                        });
                                        view! {
                                            <div style="border: 1px solid #CCC;padding: 0.5em;margin-bottom: 0.5em;">
                                                <Show when={
                                                    let status = job_step_record.status;
                                                    move || StepRecordStatus::Running==status
                                                }>
                                                    <Button onclick={on_continue} style={SharedString::from("margin-left:0.5em;")}>{"继续"}</Button>
                                                    <Button onclick={on_stop} style={SharedString::from("margin-left:0.5em;")}>{"终止"}</Button>
                                                </Show>
                                            </div>
                                        }.into_any()
                                    },
                                }
                            }
                        </Visable>
                    </div>
                }
            }
        />
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

fn render_output(output: &str) -> impl IntoView {
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
                sub_list.push(
                    view! {
                        <br/>
                    }
                    .into_any(),
                );
            }
            if !item.is_empty() {
                sub_list.push(item.to_string().into_any());
            }
        }
        list.push(view! {
            <p style={style.clone()}>
                { format!("{} {} ", log.time.format(FORMAT), log.level) }
                {sub_list}
            </p>
        });
    }
    list
}

async fn read_job_record_detail(
    detail: &RwSignal<Option<JobRecord>>,
    id: Id,
    first_time: bool,
) -> Result<JobRecord, SharedString> {
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

async fn start_read_loop(detail: &RwSignal<Option<JobRecord>>, id: Id, destroyed: &AtomicBool) {
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
    is_saving: RwSignal<bool>,
) -> Result<(), SharedString> {
    ContinueJobApi
        .lock_handler(is_saving)
        .call(&ContinueJobReq {
            record_id: record_id,
            step_record_id: step_record_id,
            success: success,
        })
        .await?;
    utils::success(SharedString::from("操作成功"));
    return Ok(());
}
