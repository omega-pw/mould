use crate::components::menu::Menu;
use crate::components::menu::MenuNode;
use crate::components::menu::State;
use crate::utils::gen_id;
use crate::SharedString;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::sync::Arc;

#[component]
pub fn SysMenu() -> impl IntoView {
    let navigate = use_navigate();
    let expanded_key = RwSignal::new(None);
    let active_key = RwSignal::new(None);
    let state = Arc::new(State {
        expanded_key: expanded_key,
        active_key: active_key,
    });
    let list = vec![
        {
            let active_key = active_key.clone();
            let key = SharedString::from(gen_id());
            let navigate = navigate.clone();
            MenuNode {
                state: state.clone(),
                key: key.clone(),
                name: String::from("环境规格"),
                action: Some(UnsyncCallback::new(move |_| {
                    navigate("/environmentSchemaList", Default::default());
                    active_key.set(Some(key.clone()));
                })),
                children: None,
            }
        },
        {
            let active_key = active_key.clone();
            let key = SharedString::from(gen_id());
            let navigate = navigate.clone();
            MenuNode {
                state: state.clone(),
                key: key.clone(),
                name: String::from("环境"),
                action: Some(UnsyncCallback::new(move |_| {
                    navigate("/environmentList", Default::default());
                    active_key.set(Some(key.clone()));
                })),
                children: None,
            }
        },
        {
            let active_key = active_key.clone();
            let key = SharedString::from(gen_id());
            let navigate = navigate.clone();
            MenuNode {
                state: state.clone(),
                key: key.clone(),
                name: String::from("任务"),
                action: Some(UnsyncCallback::new(move |_| {
                    navigate("/jobList", Default::default());
                    active_key.set(Some(key.clone()));
                })),
                children: None,
            }
        },
        {
            let active_key = active_key.clone();
            let key = SharedString::from(gen_id());
            let navigate = navigate.clone();
            MenuNode {
                state: state.clone(),
                key: key.clone(),
                name: String::from("成员"),
                action: Some(UnsyncCallback::new(move |_| {
                    navigate("/userList", Default::default());
                    active_key.set(Some(key.clone()));
                })),
                children: None,
            }
        },
    ];
    view! {
        <Menu list={list.clone()} />
    }
}
