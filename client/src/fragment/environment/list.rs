use super::super::job_record::list::JobRecordList;
use super::detail::EnvironmentDetail;
use super::edit::EnvironmentEdit;
use crate::components::button::Button;
use crate::components::button_group::ButtonGroup;
use crate::components::drawer::Drawer;
use crate::components::pagination::Pagination as PaginationComp;
use crate::sdk;
use crate::utils;
use crate::utils::fix_page_no;
use crate::utils::list_exception_view;
use crate::utils::request::ApiExt;
use crate::utils::LoadStatus;
use crate::SharedString;
use leptos::prelude::*;
use sdk::environment::delete_environment::DeleteEnvironmentApi;
use sdk::environment::delete_environment::DeleteEnvironmentReq;
use sdk::environment::query_environment::Environment;
use sdk::environment::query_environment::QueryEnvironmentApi;
use sdk::environment::query_environment::QueryEnvironmentReq;
use tihu::Id;
use tihu::Pagination;
use tihu::PrimaryKey;

#[component]
pub fn EnvironmentList() -> impl IntoView {
    let pagination: RwSignal<Pagination> = RwSignal::new(Pagination::new(0, 1, None, None));
    let list: RwSignal<Vec<Environment>> = RwSignal::new(Vec::new());
    let list_load_status: RwSignal<LoadStatus> = RwSignal::new(LoadStatus::NotStarted);
    let active_environment_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let detail_active: RwSignal<bool> = RwSignal::new(false);
    let active_detail_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let edit_active: RwSignal<bool> = RwSignal::new(false);
    let active_edit_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let record_list_active: RwSignal<bool> = RwSignal::new(false);
    let active_edit_id_clone = active_edit_id.clone();
    let edit_active_clone = edit_active.clone();
    let on_open_insert = UnsyncCallback::new(move |_: ()| {
        active_edit_id_clone.set(None);
        edit_active_clone.set(true);
    });
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let query_by_page = move |page: u64| {
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_environment_list(page, &list, &list_load_status, &pagination).await;
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
    let edit_active_clone = edit_active.clone();
    let active_edit_id_clone = active_edit_id.clone();
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_finish_save = UnsyncCallback::new(move |_pri_key: PrimaryKey| {
        edit_active_clone.set(false);
        let active_edit_id = active_edit_id_clone.clone();
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if active_edit_id.read().is_some() {
                refresh_list(&list, &list_load_status, &pagination).await;
            } else {
                query_environment_list(1, &list, &list_load_status, &pagination).await;
            }
        });
    });
    let detail_active_clone = detail_active.clone();
    let active_detail_id_clone = active_detail_id.clone();
    let on_leave_detail = UnsyncCallback::new(move |_| {
        detail_active_clone.set(false);
        active_detail_id_clone.set(None);
    });
    let edit_active_clone = edit_active.clone();
    let active_edit_id_clone = active_edit_id.clone();
    let on_leave_edit = UnsyncCallback::new(move |_| {
        edit_active_clone.set(false);
        active_edit_id_clone.set(None);
    });
    view! {
        <div class="relative width-fill height-fill" style="overflow:hidden;">
            <Show when={
                let record_list_active = record_list_active.clone();
                move || !record_list_active.get()
            }>
                <div class="width-fill height-fill border-box" style="padding:0.25em;">
                    <div class="width-fill height-fill" style="display: -webkit-box;display: flex;-webkit-box-direction: normal;-webkit-box-orient: vertical;flex-direction: column;">
                        <header style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;">
                            <div style="display:flex;justify-content: space-between;padding-bottom: 0.25em;">
                                <span style="font-weight:bold;">{"环境列表"}</span>
                                <ButtonGroup>
                                    <Button onclick={on_open_insert}>{"新增"}</Button>
                                    <Button onclick={on_query}>{"查询"}</Button>
                                </ButtonGroup>
                            </div>
                        </header>
                        <div style="-webkit-box-flex: 1;flex-basis: auto;flex-grow: 1;overflow-y: auto;">
                            { table_view(&list, &list_load_status, &pagination, &record_list_active, &active_environment_id, &detail_active, &active_detail_id, &edit_active, &active_edit_id) }
                            { list_exception_view(list, list_load_status) }
                        </div>
                        <div style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;padding-top: 0.25em;">
                            <PaginationComp pagination={pagination} onpage={on_page} />
                        </div>
                    </div>
                </div>
            </Show>
            <Drawer active={detail_active} onclickother={on_leave_detail}>
                {
                    move || {
                        match active_detail_id.get() {
                            Some(active_detail_id) => view! {
                                <EnvironmentDetail id={active_detail_id} />
                            }.into_any(),
                            None => view! {}.into_any()
                        }
                    }
                }
            </Drawer>
            <Drawer active={edit_active} onclickother={on_leave_edit}>
                {
                    let active_edit_id = active_edit_id.clone();
                    move || {
                        view! {
                            <EnvironmentEdit id={active_edit_id.get()} onsave={on_finish_save} />
                        }
                    }
                }
            </Drawer>
            {
                move || {
                    if let (true, Some(active_environment_id)) = (record_list_active.get(), active_environment_id.get()) {
                        view! {
                            <JobRecordList environment_id={active_environment_id} onclose={UnsyncCallback::new(move |_| {
                                record_list_active.set(false);
                            })}/>
                        }.into_any()
                    } else {
                        view!{}.into_any()
                    }
                }
            }
        </div>
    }
}

fn table_view(
    list: &RwSignal<Vec<Environment>>,
    list_load_status: &RwSignal<LoadStatus>,
    pagination: &RwSignal<Pagination>,
    record_list_active: &RwSignal<bool>,
    active_environment_id: &RwSignal<Option<Id>>,
    detail_active: &RwSignal<bool>,
    active_detail_id: &RwSignal<Option<Id>>,
    edit_active: &RwSignal<bool>,
    active_edit_id: &RwSignal<Option<Id>>,
) -> impl IntoView + use<> {
    let list = StoredValue::new(list.clone());
    let list_load_status = StoredValue::new(list_load_status.clone());
    let pagination = StoredValue::new(pagination.clone());
    let record_list_active = StoredValue::new(record_list_active.clone());
    let active_environment_id = StoredValue::new(active_environment_id.clone());
    let detail_active = StoredValue::new(detail_active.clone());
    let active_detail_id = StoredValue::new(active_detail_id.clone());
    let edit_active = StoredValue::new(edit_active.clone());
    let active_edit_id = StoredValue::new(active_edit_id.clone());
    return view! {
        <table class="e-table width-fill">
            <thead>
                <tr>
                    <th class="e-table-hcell">{"环境名称"}</th>
                    <th class="e-table-hcell">{"环境规格"}</th>
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
                            row_view(&list.read_value(), &list_load_status.read_value(), &pagination.read_value(), &item, &record_list_active.read_value(), &active_environment_id.read_value(), &detail_active.read_value(), &active_detail_id.read_value(), &edit_active.read_value(), &active_edit_id.read_value())
                        }
                    />
                </tbody>
            </Show>
        </table>
    };
}

fn row_view(
    list: &RwSignal<Vec<Environment>>,
    list_load_status: &RwSignal<LoadStatus>,
    pagination: &RwSignal<Pagination>,
    environment: &Environment,
    record_list_active: &RwSignal<bool>,
    active_environment_id: &RwSignal<Option<Id>>,
    detail_active: &RwSignal<bool>,
    active_detail_id: &RwSignal<Option<Id>>,
    edit_active: &RwSignal<bool>,
    active_edit_id: &RwSignal<Option<Id>>,
) -> impl IntoView + use<> {
    let detail_id = environment.id;
    let active_environment_id = active_environment_id.clone();
    let record_list_active = record_list_active.clone();
    let on_open_record_list = UnsyncCallback::new(move |_: ()| {
        active_environment_id.set(Some(detail_id));
        record_list_active.set(true);
    });
    let active_detail_id = active_detail_id.clone();
    let detail_active = detail_active.clone();
    let on_open_detail = UnsyncCallback::new(move |_: ()| {
        active_detail_id.set(Some(detail_id));
        detail_active.set(true);
    });
    let update_id = environment.id;
    let active_edit_id = active_edit_id.clone();
    let edit_active = edit_active.clone();
    let on_open_update = UnsyncCallback::new(move |_: ()| {
        active_edit_id.set(Some(update_id));
        edit_active.set(true);
    });
    let delete_id = environment.id;
    let list = list.clone();
    let list_load_status = list_load_status.clone();
    let pagination = pagination.clone();
    let on_confirm_delete = UnsyncCallback::new(move |_: ()| {
        let list = list.clone();
        let list_load_status = list_load_status.clone();
        let pagination = pagination.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let ok = utils::confirm(SharedString::from("确定删除该环境？"), None).await;
            if ok {
                let list = list.clone();
                let list_load_status = list_load_status.clone();
                let pagination = pagination.clone();
                delete_environment(delete_id, &list, &list_load_status, &pagination)
                    .await
                    .ok();
            }
        });
    });
    return view! {
        <tr class="e-table-row">
            <td class="e-table-cell align-center">{environment.name.clone()}</td>
            <td class="e-table-cell align-center">{environment.environment_schema_name.clone()}</td>
            <td class="e-table-cell align-center">
                <ButtonGroup>
                    <Button onclick={on_open_record_list}>{"执行记录"}</Button>
                    <Button onclick={on_open_detail}>{"详情"}</Button>
                    <Button onclick={on_open_update}>{"编辑"}</Button>
                    <Button onclick={on_confirm_delete}>{"删除"}</Button>
                </ButtonGroup>
            </td>
        </tr>
    };
}

fn clear_list(list: &RwSignal<Vec<Environment>>, pagination: &RwSignal<Pagination>) {
    pagination.set(Pagination::new(0, 1, None, None));
    list.set(Vec::new());
}

async fn query_environment_list(
    page_no: u64,
    list: &RwSignal<Vec<Environment>>,
    list_load_status: &RwSignal<LoadStatus>,
    pagination: &RwSignal<Pagination>,
) {
    let params = QueryEnvironmentReq {
        page_no: Some(page_no),
        ..QueryEnvironmentReq::empty()
    };
    list_load_status.set(LoadStatus::Loading);
    let ret = QueryEnvironmentApi.call(&params).await;
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
    list: &RwSignal<Vec<Environment>>,
    list_load_status: &RwSignal<LoadStatus>,
    pagination: &RwSignal<Pagination>,
) {
    let new_page_no = fix_page_no(pagination, list);
    query_environment_list(new_page_no, list, list_load_status, pagination).await;
}

async fn delete_environment(
    id: Id,
    list: &RwSignal<Vec<Environment>>,
    list_load_status: &RwSignal<LoadStatus>,
    pagination: &RwSignal<Pagination>,
) -> Result<(), SharedString> {
    let params = DeleteEnvironmentReq { id: id };
    DeleteEnvironmentApi.call(&params).await?;
    remove_environment(id, list);
    utils::success(SharedString::from("删除成功"));
    let list = list.clone();
    let list_load_status = list_load_status.clone();
    let pagination = pagination.clone();
    wasm_bindgen_futures::spawn_local(async move {
        refresh_list(&list, &list_load_status, &pagination).await;
    });
    return Ok(());
}

fn remove_environment(id: Id, list: &RwSignal<Vec<Environment>>) {
    list.write().retain(|item| {
        return item.id != id;
    });
}
