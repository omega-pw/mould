use super::detail::UserDetail;
use super::invite::InviteEdit;
use crate::components::button::Button;
use crate::components::button_group::ButtonGroup;
use crate::components::drawer::Drawer;
use crate::components::image::Image;
use crate::components::pagination::Pagination as PaginationComp;
use crate::sdk;
use crate::utils::format_time_local;
use crate::utils::list_exception_view;
use crate::utils::request::ApiExt;
use crate::utils::LoadStatus;
use crate::SharedString;
use leptos::prelude::*;
use sdk::user::query_user::QueryUserApi;
use sdk::user::query_user::QueryUserReq;
use sdk::user::query_user::User;
use tihu::Id;
use tihu::Pagination;
use tihu::PrimaryKey;

#[component]
pub fn UserList() -> impl IntoView {
    let pagination: RwSignal<Pagination> = RwSignal::new(Pagination::new(0, 1, None, None));
    let list: RwSignal<Vec<User>> = RwSignal::new(Vec::new());
    let list_load_status: RwSignal<LoadStatus> = RwSignal::new(LoadStatus::NotStarted);
    let detail_active: RwSignal<bool> = RwSignal::new(false);
    let active_detail_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let invite_active: RwSignal<bool> = RwSignal::new(false);
    let invite_active_clone = invite_active.clone();
    let on_open_invite = UnsyncCallback::new(move |_: ()| {
        invite_active_clone.set(true);
    });
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let query_by_page = move |page: u64| {
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_user_list(page, &list, &list_load_status, &pagination).await;
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
    let invite_active_clone = invite_active.clone();
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_finish_invite = UnsyncCallback::new(move |_pri_key: PrimaryKey| {
        invite_active_clone.set(false);
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_user_list(1, &list, &list_load_status, &pagination).await;
        });
    });
    let detail_active_clone = detail_active.clone();
    let active_detail_id_clone = active_detail_id.clone();
    let on_leave_detail = UnsyncCallback::new(move |_| {
        detail_active_clone.set(false);
        active_detail_id_clone.set(None);
    });
    let invite_active_clone = invite_active.clone();
    let on_leave_invite = UnsyncCallback::new(move |_| {
        invite_active_clone.set(false);
    });
    view! {
        <div class="relative width-fill height-fill" style="overflow:hidden;">
            <div class="width-fill height-fill border-box" style="padding:0.25em;">
                <div class="width-fill height-fill" style="display: -webkit-box;display: flex;-webkit-box-direction: normal;-webkit-box-orient: vertical;flex-direction: column;">
                    <header style="-webkit-box-flex: 0;flex-basis: auto;flex-grow: 0;">
                        <div class="align-right" style="padding-bottom: 0.25em;">
                            <ButtonGroup>
                                <Button onclick={on_open_invite}>{"邀请"}</Button>
                                <Button onclick={on_query}>{"查询"}</Button>
                            </ButtonGroup>
                        </div>
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
                                <UserDetail id={active_detail_id} />
                            }.into_any(),
                            None => view! {}.into_any()
                        }
                    }
                }
            </Drawer>
            <Drawer active={invite_active} onclickother={on_leave_invite}>
                <InviteEdit onsave={on_finish_invite} />
            </Drawer>
        </div>
    }
}

fn table_view(
    list: &RwSignal<Vec<User>>,
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
                    <th class="e-table-hcell">{"用户来源"}</th>
                    <th class="e-table-hcell">{"名称"}</th>
                    <th class="e-table-hcell">{"头像"}</th>
                    <th class="e-table-hcell">{"创建时间"}</th>
                    <th class="e-table-hcell">{"更新时间"}</th>
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
    user: &User,
    detail_active: &RwSignal<bool>,
    active_detail_id: &RwSignal<Option<Id>>,
) -> impl IntoView + use<> {
    let detail_id = user.id;
    let active_detail_id = active_detail_id.clone();
    let detail_active = detail_active.clone();
    let on_open_detail = UnsyncCallback::new(move |_: ()| {
        active_detail_id.set(Some(detail_id));
        detail_active.set(true);
    });
    return view! {
        <tr class="e-table-row">
            <td class="e-table-cell align-center">{user.user_source.to_string()}</td>
            <td class="e-table-cell align-center">{user.name.clone()}</td>
            <td class="e-table-cell align-center">
                {
                    if let Some(avatar_url) = user.avatar_url.as_ref() {
                        view! {
                            <Image src={SharedString::from(avatar_url.clone())} style="max-height: 3em;"/>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }
            </td>
            <td class="e-table-cell align-center">{ format_time_local(&user.created_time) }</td>
            <td class="e-table-cell align-center">{ format_time_local(&user.last_modified_time) }</td>
            <td class="e-table-cell align-center">
                <ButtonGroup>
                    <Button onclick={on_open_detail}>{"详情"}</Button>
                </ButtonGroup>
            </td>
        </tr>
    };
}

fn clear_list(list: &RwSignal<Vec<User>>, pagination: &RwSignal<Pagination>) {
    pagination.set(Pagination::new(0, 1, None, None));
    list.set(Vec::new());
}

async fn query_user_list(
    page_no: u64,
    list: &RwSignal<Vec<User>>,
    list_load_status: &RwSignal<LoadStatus>,
    pagination: &RwSignal<Pagination>,
) {
    let params = QueryUserReq {
        page_no: Some(page_no),
        ..QueryUserReq::empty()
    };
    list_load_status.set(LoadStatus::Loading);
    let ret = QueryUserApi.call(&params).await;
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
