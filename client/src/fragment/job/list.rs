use super::super::job_record::list::JobRecordList;
use super::detail::JobDetail;
use super::edit::JobEdit;
use super::start_job::StartJob;
use crate::components::button::Button;
use crate::components::button_group::ButtonGroup;
use crate::components::drawer::Drawer;
use crate::components::pagination::Pagination as PaginationComp;
use crate::components::r#if::If;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::utils::LoadStatus;
use crate::LightString;
use sdk::job::delete_job::DeleteJobApi;
use sdk::job::delete_job::DeleteJobReq;
use sdk::job::query_job::Job;
use sdk::job::query_job::QueryJobApi;
use sdk::job::query_job::QueryJobReq;
use std::ops::Deref;
use tihu::Id;
use tihu::Pagination;
use tihu::PrimaryKey;
use yew::prelude::*;

#[function_component]
pub fn JobList() -> Html {
    let pagination: UseStateHandle<Pagination> = use_state(|| Pagination::new(0, 1, None, None));
    let list: UseStateHandle<Vec<Job>> = use_state(|| Vec::new());
    let list_load_status: UseStateHandle<LoadStatus> = use_state(|| LoadStatus::NotStarted);
    let active_job_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let detail_active: UseStateHandle<bool> = use_state(|| false);
    let active_detail_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let edit_active: UseStateHandle<bool> = use_state(|| false);
    let start_active: UseStateHandle<bool> = use_state(|| false);
    let record_list_active: UseStateHandle<bool> = use_state(|| false);
    let active_start_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let active_edit_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let active_edit_id_clone = active_edit_id.clone();
    let edit_active_clone = edit_active.clone();
    let on_open_insert = Callback::from(move |_: ()| {
        active_edit_id_clone.set(None);
        edit_active_clone.set(true);
    });
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_query = Callback::from(move |_| {
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_job_list(1, &list, &list_load_status, &pagination).await;
        });
    });
    let on_query_clone = on_query.clone();
    use_effect_with((), move |_| {
        on_query_clone.emit(());
        || ()
    });
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_page = Callback::from(move |page: u64| {
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_job_list(page, &list, &list_load_status, &pagination).await;
        });
    });
    let edit_active_clone = edit_active.clone();
    let active_edit_id_clone = active_edit_id.clone();
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_finish_save = Callback::from(move |_pri_key: PrimaryKey| {
        edit_active_clone.set(false);
        let active_edit_id = active_edit_id_clone.clone();
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if active_edit_id.is_some() {
                refresh_list(&list, &list_load_status, &pagination).await;
            } else {
                query_job_list(1, &list, &list_load_status, &pagination).await;
            }
        });
    });
    let detail_active_clone = detail_active.clone();
    let active_detail_id_clone = active_detail_id.clone();
    let on_leave_detail = Callback::from(move |_| {
        detail_active_clone.set(false);
        active_detail_id_clone.set(None);
    });
    let start_active_clone = start_active.clone();
    let active_start_id_clone = active_start_id.clone();
    let on_leave_start = Callback::from(move |_| {
        start_active_clone.set(false);
        active_start_id_clone.set(None);
    });
    let edit_active_clone = edit_active.clone();
    let active_edit_id_clone = active_edit_id.clone();
    let on_leave_edit = Callback::from(move |_| {
        edit_active_clone.set(false);
        active_edit_id_clone.set(None);
    });
    html! {
        <div class="relative width-fill height-fill" style="overflow:hidden;">
            <If condition={!*record_list_active}>
                <div class="width-fill height-fill border-box" style="padding:0.25em;">
                    <div class="width-fill height-fill" style="display: -webkit-box;display: flex;-webkit-box-direction: normal;-webkit-box-orient: vertical;flex-direction: column;">
                        <header style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;">
                            <div style="display:flex;justify-content: space-between;padding-bottom: 0.25em;">
                                <span style="font-weight:bold;">{"任务列表"}</span>
                                <ButtonGroup>
                                    <Button onclick={on_open_insert}>{"新增"}</Button>
                                    <Button onclick={on_query}>{"查询"}</Button>
                                </ButtonGroup>
                            </div>
                        </header>
                        <div style="-webkit-box-flex: 1;flex-basis: auto;flex-grow: 1;overflow-y: auto;">
                            { table_view(&list, &list_load_status, &pagination, &record_list_active, &active_job_id, &detail_active, &active_detail_id, &start_active, &active_start_id, &edit_active, &active_edit_id) }
                            { list_exception_view(list.is_empty(), list_load_status.deref().clone()) }
                        </div>
                        <div style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;padding-top: 0.25em;">
                            <PaginationComp pagination={pagination.deref().clone()} onpage={on_page} />
                        </div>
                    </div>
                </div>
            </If>
            <Drawer active={*detail_active} onclickother={on_leave_detail}>
                {
                    match active_detail_id.as_ref() {
                        Some(active_detail_id) => html! {
                            <JobDetail id={*active_detail_id} />
                        },
                        None => html! {}
                    }
                }
            </Drawer>
            <Drawer active={*start_active} onclickother={on_leave_start}>
                {
                    if let Some(active_start_id) = active_start_id.as_ref() {
                        html! {
                            <StartJob id={*active_start_id} />
                        }
                    } else {
                        html!{}
                    }
                }
            </Drawer>
            <Drawer active={*edit_active} onclickother={on_leave_edit}>
                <JobEdit id={active_edit_id.deref().clone()} onsave={on_finish_save} />
            </Drawer>
            {
                if let (true, Some(active_job_id)) = (*record_list_active, active_job_id.as_ref()) {
                    html! {
                        <JobRecordList job_id={*active_job_id} onclose={Callback::from(move |_| {
                            record_list_active.set(false);
                        })}/>
                    }
                } else {
                    html!{}
                }
            }
        </div>
    }
}

fn table_view(
    list: &UseStateHandle<Vec<Job>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    pagination: &UseStateHandle<Pagination>,
    record_list_active: &UseStateHandle<bool>,
    active_job_id: &UseStateHandle<Option<Id>>,
    detail_active: &UseStateHandle<bool>,
    active_detail_id: &UseStateHandle<Option<Id>>,
    start_active: &UseStateHandle<bool>,
    active_start_id: &UseStateHandle<Option<Id>>,
    edit_active: &UseStateHandle<bool>,
    active_edit_id: &UseStateHandle<Option<Id>>,
) -> Html {
    return html! {
        <table class="e-table width-fill">
            <thead>
                <tr>
                    <th class="e-table-hcell">{"任务名称"}</th>
                    <th class="e-table-hcell">{"环境规格"}</th>
                    <th class="e-table-hcell">{"操作"}</th>
                </tr>
            </thead>
            {
                if list_load_status.deref() == &LoadStatus::LoadOk || list_load_status.deref() == &LoadStatus::Loading {
                    html! {
                        <tbody>
                            {
                                for list.iter().map(|item| {
                                    row_view(list, list_load_status, pagination, item, record_list_active, active_job_id, detail_active, active_detail_id, start_active, active_start_id, edit_active, active_edit_id)
                                })
                            }
                        </tbody>
                    }
                } else {
                    html! {}
                }
            }
        </table>
    };
}

fn row_view(
    list: &UseStateHandle<Vec<Job>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    pagination: &UseStateHandle<Pagination>,
    job: &Job,
    record_list_active: &UseStateHandle<bool>,
    active_job_id: &UseStateHandle<Option<Id>>,
    detail_active: &UseStateHandle<bool>,
    active_detail_id: &UseStateHandle<Option<Id>>,
    start_active: &UseStateHandle<bool>,
    active_start_id: &UseStateHandle<Option<Id>>,
    edit_active: &UseStateHandle<bool>,
    active_edit_id: &UseStateHandle<Option<Id>>,
) -> Html {
    let detail_id = job.id;
    let active_job_id = active_job_id.clone();
    let record_list_active = record_list_active.clone();
    let on_open_record_list = Callback::from(move |_: ()| {
        active_job_id.set(Some(detail_id));
        record_list_active.set(true);
    });
    let active_detail_id = active_detail_id.clone();
    let detail_active = detail_active.clone();
    let on_open_detail = Callback::from(move |_: ()| {
        active_detail_id.set(Some(detail_id));
        detail_active.set(true);
    });
    let start_active = start_active.clone();
    let active_start_id = active_start_id.clone();
    let on_start = Callback::from(move |_: ()| {
        active_start_id.set(Some(detail_id));
        start_active.set(true);
    });
    let update_id = job.id;
    let active_edit_id = active_edit_id.clone();
    let edit_active = edit_active.clone();
    let on_open_update = Callback::from(move |_: ()| {
        active_edit_id.set(Some(update_id));
        edit_active.set(true);
    });
    let delete_id = job.id;
    let list = list.clone();
    let list_load_status = list_load_status.clone();
    let pagination = pagination.clone();
    let on_confirm_delete = Callback::from(move |_: ()| {
        let list = list.clone();
        let list_load_status = list_load_status.clone();
        let pagination = pagination.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let ok = utils::confirm(LightString::from("确定删除该任务？"), None).await;
            if ok {
                let list = list.clone();
                let list_load_status = list_load_status.clone();
                let pagination = pagination.clone();
                delete_job(delete_id, &list, &list_load_status, &pagination)
                    .await
                    .ok();
            }
        });
    });
    return html! {
        <tr class="e-table-row">
            <td class="e-table-cell align-center">{job.name.clone()}</td>
            <td class="e-table-cell align-center">{job.environment_schema_name.clone()}</td>
            <td class="e-table-cell align-center">
                <ButtonGroup>
                    <Button onclick={on_open_record_list}>{"执行记录"}</Button>
                    <Button onclick={on_open_detail}>{"详情"}</Button>
                    <Button onclick={on_start}>{"运行"}</Button>
                    <Button onclick={on_open_update}>{"编辑"}</Button>
                    <Button onclick={on_confirm_delete}>{"删除"}</Button>
                </ButtonGroup>
            </td>
        </tr>
    };
}

fn list_exception_view(is_empty: bool, list_load_status: LoadStatus) -> Html {
    return html! {
        match list_load_status {
            LoadStatus::LoadFailed => {
                html! {
                    <p class="align-center">{"列表加载失败"}</p>
                }
            },
            LoadStatus::LoadOk => {
                if is_empty {
                    html! {
                        <p class="align-center">{"列表数据为空"}</p>
                    }
                } else {
                    html! {}
                }
            },
            _ => html! {}
        }
    };
}

fn clear_list(list: &UseStateHandle<Vec<Job>>, pagination: &UseStateHandle<Pagination>) {
    pagination.set(Pagination::new(0, 1, None, None));
    list.set(Vec::new());
}

async fn query_job_list(
    page_no: u64,
    list: &UseStateHandle<Vec<Job>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    pagination: &UseStateHandle<Pagination>,
) {
    let params = QueryJobReq {
        page_no: Some(page_no),
        ..QueryJobReq::empty()
    };
    list_load_status.set(LoadStatus::Loading);
    let ret = QueryJobApi.call(&params).await;
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

async fn refresh_list(
    list: &UseStateHandle<Vec<Job>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    pagination: &UseStateHandle<Pagination>,
) {
    let page_no = if list.is_empty() {
        pagination.page_no - 1
    } else {
        pagination.page_no
    };
    query_job_list(page_no.max(1), list, list_load_status, pagination).await;
}

async fn delete_job(
    id: Id,
    list: &UseStateHandle<Vec<Job>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    pagination: &UseStateHandle<Pagination>,
) -> Result<(), LightString> {
    let params = DeleteJobReq { id: id };
    DeleteJobApi.call(&params).await?;
    remove_job(id, list);
    utils::success(LightString::from("删除成功"));
    let list = list.clone();
    let list_load_status = list_load_status.clone();
    let pagination = pagination.clone();
    wasm_bindgen_futures::spawn_local(async move {
        refresh_list(&list, &list_load_status, &pagination).await;
    });
    return Ok(());
}

fn remove_job(id: Id, list: &UseStateHandle<Vec<Job>>) {
    let new_list: Vec<_> = list
        .iter()
        .filter_map(|item| {
            if item.id == id {
                None
            } else {
                Some(item.clone())
            }
        })
        .collect();
    list.set(new_list);
}
