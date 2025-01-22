use super::detail::JobRecordDetail;
use crate::components::button::Button;
use crate::components::button_group::ButtonGroup;
use crate::components::drawer::Drawer;
use crate::components::pagination::Pagination as PaginationComp;
use crate::sdk;
use crate::utils::request::ApiExt;
use crate::utils::LoadStatus;
use sdk::job_record::query_job_record::JobRecord;
use sdk::job_record::query_job_record::QueryJobRecordApi;
use sdk::job_record::query_job_record::QueryJobRecordReq;
use std::ops::Deref;
use tihu::datetime_format::FORMAT;
use tihu::Id;
use tihu::Pagination;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub job_id: Option<Id>,
    #[prop_or_default]
    pub environment_id: Option<Id>,
    #[prop_or_default]
    pub onclose: Callback<()>,
}

#[function_component]
pub fn JobRecordList(props: &Props) -> Html {
    let pagination: UseStateHandle<Pagination> = use_state(|| Pagination::new(0, 1, None, None));
    let list: UseStateHandle<Vec<JobRecord>> = use_state(|| Vec::new());
    let list_load_status: UseStateHandle<LoadStatus> = use_state(|| LoadStatus::NotStarted);
    let detail_active: UseStateHandle<bool> = use_state(|| false);
    let active_detail_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let props_clone = props.clone();
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_query = Callback::from(move |_| {
        let props = props_clone.clone();
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_job_record_list(&props, 1, &list, &list_load_status, &pagination).await;
        });
    });
    let on_query_clone = on_query.clone();
    use_effect_with((), move |_| {
        on_query_clone.emit(());
        || ()
    });
    let props_clone = props.clone();
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_page = Callback::from(move |page: u64| {
        let props = props_clone.clone();
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_job_record_list(&props, page, &list, &list_load_status, &pagination).await;
        });
    });

    let detail_active_clone = detail_active.clone();
    let active_detail_id_clone = active_detail_id.clone();
    let on_leave_detail = Callback::from(move |_| {
        detail_active_clone.set(false);
        active_detail_id_clone.set(None);
    });
    html! {
        <div class="relative width-fill height-fill" style="overflow:hidden;">
            <div class="width-fill height-fill border-box" style="padding:0.25em;">
                <div class="width-fill height-fill" style="display: -webkit-box;display: flex;-webkit-box-direction: normal;-webkit-box-orient: vertical;flex-direction: column;">
                    <header style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;display:flex;justify-content: space-between;padding-bottom: 0.25em;">
                        <span style="font-weight:bold;">{"执行记录列表"}</span>
                        <ButtonGroup>
                            <Button onclick={on_query}>{"查询"}</Button>
                            <Button onclick={props.onclose.clone()}>{"返回"}</Button>
                        </ButtonGroup>
                    </header>
                    <div style="-webkit-box-flex: 1;flex-basis: auto;flex-grow: 1;overflow-y: auto;">
                        { table_view(&list, &list_load_status, &detail_active, &active_detail_id) }
                        { list_exception_view(list.is_empty(), list_load_status.deref().clone()) }
                    </div>
                    <div style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;padding-top: 0.25em;">
                        <PaginationComp pagination={pagination.deref().clone()} onpage={on_page} />
                    </div>
                </div>
            </div>
            <Drawer active={*detail_active} onclickother={on_leave_detail}>
                {
                    match active_detail_id.as_ref() {
                        Some(active_detail_id) => html! {
                            <JobRecordDetail id={*active_detail_id} />
                        },
                        None => html! {}
                    }
                }
            </Drawer>
        </div>
    }
}

fn table_view(
    list: &UseStateHandle<Vec<JobRecord>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    detail_active: &UseStateHandle<bool>,
    active_detail_id: &UseStateHandle<Option<Id>>,
) -> Html {
    return html! {
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
            {
                if list_load_status.deref() == &LoadStatus::LoadOk || list_load_status.deref() == &LoadStatus::Loading {
                    html! {
                        <tbody>
                            {
                                for list.iter().map(|item| {
                                    row_view(item, detail_active, active_detail_id)
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
    job_record: &JobRecord,
    detail_active: &UseStateHandle<bool>,
    active_detail_id: &UseStateHandle<Option<Id>>,
) -> Html {
    let detail_id = job_record.id;
    let active_detail_id = active_detail_id.clone();
    let detail_active = detail_active.clone();
    let on_open_detail = Callback::from(move |_: ()| {
        active_detail_id.set(Some(detail_id));
        detail_active.set(true);
    });
    return html! {
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

fn clear_list(list: &UseStateHandle<Vec<JobRecord>>, pagination: &UseStateHandle<Pagination>) {
    pagination.set(Pagination::new(0, 1, None, None));
    list.set(Vec::new());
}

async fn query_job_record_list(
    props: &Props,
    page_no: u64,
    list: &UseStateHandle<Vec<JobRecord>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    pagination: &UseStateHandle<Pagination>,
) {
    let params = QueryJobRecordReq {
        page_no: Some(page_no),
        job_id: props.job_id.clone(),
        environment_id: props.environment_id.clone(),
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
