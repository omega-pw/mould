use super::detail::UserDetail;
use super::invite::InviteEdit;
use crate::components::button::Button;
use crate::components::button_group::ButtonGroup;
use crate::components::drawer::Drawer;
use crate::components::image::Image;
use crate::components::pagination::Pagination as PaginationComp;
use crate::sdk;
use crate::utils::format_time_local;
use crate::utils::request::ApiExt;
use crate::utils::LoadStatus;
use crate::LightString;
use sdk::user::query_user::QueryUserApi;
use sdk::user::query_user::QueryUserReq;
use sdk::user::query_user::User;
use std::ops::Deref;
use tihu::Id;
use tihu::Pagination;
use tihu::PrimaryKey;
use yew::prelude::*;

#[function_component]
pub fn UserList() -> Html {
    let pagination: UseStateHandle<Pagination> = use_state(|| Pagination::new(0, 1, None, None));
    let list: UseStateHandle<Vec<User>> = use_state(|| Vec::new());
    let list_load_status: UseStateHandle<LoadStatus> = use_state(|| LoadStatus::NotStarted);
    let detail_active: UseStateHandle<bool> = use_state(|| false);
    let active_detail_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let invite_active: UseStateHandle<bool> = use_state(|| false);
    let invite_active_clone = invite_active.clone();
    let on_open_invite = Callback::from(move |_: ()| {
        invite_active_clone.set(true);
    });
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_query = Callback::from(move |_| {
        let list = list_clone.clone();
        let list_load_status = list_load_status_clone.clone();
        let pagination = pagination_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            query_user_list(1, &list, &list_load_status, &pagination).await;
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
            query_user_list(page, &list, &list_load_status, &pagination).await;
        });
    });
    let invite_active_clone = invite_active.clone();
    let list_clone = list.clone();
    let list_load_status_clone = list_load_status.clone();
    let pagination_clone = pagination.clone();
    let on_finish_invite = Callback::from(move |_pri_key: PrimaryKey| {
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
    let on_leave_detail = Callback::from(move |_| {
        detail_active_clone.set(false);
        active_detail_id_clone.set(None);
    });
    let invite_active_clone = invite_active.clone();
    let on_leave_invite = Callback::from(move |_| {
        invite_active_clone.set(false);
    });
    html! {
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
                            <UserDetail id={*active_detail_id} />
                        },
                        None => html! {}
                    }
                }
            </Drawer>
            <Drawer active={*invite_active} onclickother={on_leave_invite}>
                <InviteEdit onsave={on_finish_invite} />
            </Drawer>
        </div>
    }
}

fn table_view(
    list: &UseStateHandle<Vec<User>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    detail_active: &UseStateHandle<bool>,
    active_detail_id: &UseStateHandle<Option<Id>>,
) -> Html {
    return html! {
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
    user: &User,
    detail_active: &UseStateHandle<bool>,
    active_detail_id: &UseStateHandle<Option<Id>>,
) -> Html {
    let detail_id = user.id;
    let active_detail_id = active_detail_id.clone();
    let detail_active = detail_active.clone();
    let on_open_detail = Callback::from(move |_: ()| {
        active_detail_id.set(Some(detail_id));
        detail_active.set(true);
    });
    return html! {
        <tr class="e-table-row">
            <td class="e-table-cell align-center">{&user.user_source.to_string()}</td>
            <td class="e-table-cell align-center">{&user.name}</td>
            <td class="e-table-cell align-center">
                if let Some(avatar_url) = user.avatar_url.as_ref() {
                    <Image src={LightString::from(avatar_url.clone())} style="max-height: 3em;"/>
                }
            </td>
            <td class="e-table-cell align-center">{ html!{&format_time_local(&user.created_time)} }</td>
            <td class="e-table-cell align-center">{ html!{&format_time_local(&user.last_modified_time)} }</td>
            <td class="e-table-cell align-center">
                <ButtonGroup>
                    <Button onclick={on_open_detail}>{"详情"}</Button>
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

fn clear_list(list: &UseStateHandle<Vec<User>>, pagination: &UseStateHandle<Pagination>) {
    pagination.set(Pagination::new(0, 1, None, None));
    list.set(Vec::new());
}

async fn query_user_list(
    page_no: u64,
    list: &UseStateHandle<Vec<User>>,
    list_load_status: &UseStateHandle<LoadStatus>,
    pagination: &UseStateHandle<Pagination>,
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
