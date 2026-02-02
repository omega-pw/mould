use super::detail::JobRecordDetail;
use crate::components::button::Button;
use crate::components::button_group::ButtonGroup;
use crate::components::drawer::Drawer;
use crate::components::pagination::Pagination as PaginationComp;
use crate::sdk;
use crate::utils::list_exception_view;
use crate::utils::request::ApiExt;
use crate::utils::LoadStatus;
use leptos::prelude::*;
use leptos::Params;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use sdk::job_record::query_job_record::JobRecord;
use sdk::job_record::query_job_record::QueryJobRecordApi;
use sdk::job_record::query_job_record::QueryJobRecordReq;
use tihu::datetime_format::FORMAT;
use tihu::Id;
use tihu::Pagination;

#[derive(Params, Clone, PartialEq)]
struct JobParams {
    job_id: Option<Id>,
}

#[component]
pub fn JobRecordListByJob() -> impl IntoView {
    let job_params = use_params::<JobParams>();
    move || {
        let job_params = job_params.get();
        match job_params {
            Ok(job_params) => {
                if let Some(job_id) = job_params.job_id {
                    view! {
                        <JobRecordList job_id={job_id}/>
                    }
                    .into_any()
                } else {
                    view! {
                        <div>
                            {"参数错误: job_id为空"}
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

#[derive(Params, Clone, PartialEq)]
struct EnvironmentParams {
    environment_id: Option<Id>,
}

#[component]
pub fn JobRecordListByEnvironment() -> impl IntoView {
    let environment_params = use_params::<EnvironmentParams>();
    move || {
        let environment_params = environment_params.get();
        match environment_params {
            Ok(environment_params) => {
                if let Some(environment_id) = environment_params.environment_id {
                    view! {
                        <JobRecordList environment_id={environment_id}/>
                    }
                    .into_any()
                } else {
                    view! {
                        <div>
                            {"参数错误: environment_id为空"}
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
pub fn JobRecordList(
    #[prop(optional)] job_id: Option<Id>,
    #[prop(optional)] environment_id: Option<Id>,
    #[prop(into, default = None)] onclose: Option<UnsyncCallback<()>>,
) -> impl IntoView {
    let pagination: RwSignal<Pagination> = RwSignal::new(Pagination::new(0, 1, None, None));
    let list: RwSignal<Vec<JobRecord>> = RwSignal::new(Vec::new());
    let list_load_status: RwSignal<LoadStatus> = RwSignal::new(LoadStatus::NotStarted);
    let detail_active: RwSignal<bool> = RwSignal::new(false);
    let active_detail_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let query_by_page = move |page: u64| {
        let job_id = job_id.clone();
        let environment_id = environment_id.clone();
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_job_record_list(
                job_id,
                environment_id,
                page,
                &list,
                &list_load_status,
                &pagination,
            )
            .await;
        });
    };
    let on_query = {
        let query_by_page = query_by_page.clone();
        UnsyncCallback::new(move |_| {
            query_by_page(1);
        })
    };
    query_by_page(1);
    let on_page = UnsyncCallback::new(query_by_page);
    let detail_active_clone = detail_active.clone();
    let active_detail_id_clone = active_detail_id.clone();
    let on_leave_detail = UnsyncCallback::new(move |_| {
        detail_active_clone.set(false);
        active_detail_id_clone.set(None);
    });
    view! {
        <div class="relative width-fill height-fill" style="overflow:hidden;">
            <div class="width-fill height-fill border-box" style="padding:0.25em;">
                <div class="width-fill height-fill" style="display: -webkit-box;display: flex;-webkit-box-direction: normal;-webkit-box-orient: vertical;flex-direction: column;">
                    <header style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;display:flex;justify-content: space-between;padding-bottom: 0.25em;">
                        <span style="font-weight:bold;">{"执行记录列表"}</span>
                        <ButtonGroup>
                            <Button onclick={on_query}>{"查询"}</Button>
                            <Button onclick={onclose}>{"返回"}</Button>
                        </ButtonGroup>
                    </header>
                    <div style="-webkit-box-flex: 1;flex-basis: auto;flex-grow: 1;overflow-y: auto;">
                        { table_view(&list, &list_load_status, &detail_active, &active_detail_id) }
                        { list_exception_view(list, list_load_status) }
                    </div>
                    <div style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;padding-top: 0.25em;">
                        <PaginationComp pagination={pagination} onpage={on_page} />
                    </div>
                </div>
            </div>
            <Drawer active={detail_active} onclickother={on_leave_detail}>
                {
                    move || {
                        match active_detail_id.get() {
                            Some(active_detail_id) => view! {
                                <JobRecordDetail id={active_detail_id} />
                            }.into_any(),
                            None => view! {}.into_any()
                        }
                    }
                }
            </Drawer>
        </div>
    }
}

fn table_view(
    list: &RwSignal<Vec<JobRecord>>,
    list_load_status: &RwSignal<LoadStatus>,
    detail_active: &RwSignal<bool>,
    active_detail_id: &RwSignal<Option<Id>>,
) -> impl IntoView + use<> {
    let list = StoredValue::new(list.clone());
    let list_load_status = StoredValue::new(list_load_status.clone());
    let detail_active = StoredValue::new(detail_active.clone());
    let active_detail_id = StoredValue::new(active_detail_id.clone());
    return view! {
        <table class="e-table width-fill">
            <thead>
                <tr>
                    <th class="e-table-hcell">{"任务名称"}</th>
                    <th class="e-table-hcell">{"环境名称"}</th>
                    <th class="e-table-hcell">{"执行状态"}</th>
                    <th class="e-table-hcell">{"创建时间"}</th>
                    <th class="e-table-hcell">{"操作"}</th>
                </tr>
            </thead>
            <Show when=move || {
                let list_load_status = list_load_status.read_value().get();
                LoadStatus::LoadOk == list_load_status || LoadStatus::Loading == list_load_status
            }>
                <tbody>
                    <For
                        each={
                            let list = list.clone();
                            move || { list.read_value().get() }
                        }
                        key=|item| { item.id.clone() }
                        children=move |item| {
                            row_view(&item, &detail_active.read_value(), &active_detail_id.read_value())
                        }
                    />
                </tbody>
            </Show>
        </table>
    };
}

fn row_view(
    job_record: &JobRecord,
    detail_active: &RwSignal<bool>,
    active_detail_id: &RwSignal<Option<Id>>,
) -> impl IntoView + use<> {
    let detail_id = job_record.id;
    let active_detail_id = active_detail_id.clone();
    let detail_active = detail_active.clone();
    let on_open_detail = UnsyncCallback::new(move |_: ()| {
        active_detail_id.set(Some(detail_id));
        detail_active.set(true);
    });
    return view! {
        <tr class="e-table-row">
            <td class="e-table-cell align-center">{job_record.job_name.clone()}</td>
            <td class="e-table-cell align-center">{job_record.environment_name.clone()}</td>
            <td class="e-table-cell align-center">{job_record.status.to_string()}</td>
            <td class="e-table-cell align-center">{format!("{}", job_record.created_time.format(FORMAT))}</td>
            <td class="e-table-cell align-center">
                <Button onclick={on_open_detail}>{"详情"}</Button>
            </td>
        </tr>
    };
}

fn clear_list(list: &RwSignal<Vec<JobRecord>>, pagination: &RwSignal<Pagination>) {
    pagination.set(Pagination::new(0, 1, None, None));
    list.set(Vec::new());
}

async fn query_job_record_list(
    job_id: Option<Id>,
    environment_id: Option<Id>,
    page_no: u64,
    list: &RwSignal<Vec<JobRecord>>,
    list_load_status: &RwSignal<LoadStatus>,
    pagination: &RwSignal<Pagination>,
) {
    let params = QueryJobRecordReq {
        page_no: Some(page_no),
        job_id: job_id.clone(),
        environment_id: environment_id.clone(),
        ..QueryJobRecordReq::empty()
    };
    list_load_status.set(LoadStatus::Loading);
    let ret = QueryJobRecordApi.call(&params).await;
    match ret {
        Err(err) => {
            log::error!("{}", err);
            list_load_status.set(LoadStatus::LoadFailed);
            clear_list(list, pagination);
        }
        Ok(pagination_list) => {
            list_load_status.set(LoadStatus::LoadOk);
            list.set(pagination_list.list);
            pagination.set(pagination_list.pagination);
        }
    }
}
