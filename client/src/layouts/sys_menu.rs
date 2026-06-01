use crate::components::menu::Menu;
use crate::components::menu::MenuNode;
use crate::components::menu::State;
use crate::utils::gen_id;
use crate::Key;
use crate::SharedString;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RouteAction {
    pub route: SharedString,
    pub permission: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Action {
    Route(RouteAction),
    Children(Vec<MenuItem>),
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub key: Key,
    pub name: String,
    pub action: Action,
}

fn get_menus() -> Vec<MenuItem> {
    let menus: Vec<MenuItem> = vec![
        MenuItem {
            key: SharedString::from(gen_id()),
            name: String::from("环境规格"),
            action: Action::Route(RouteAction {
                route: SharedString::from("/environmentSchemaList"),
                permission: None,
            }),
        },
        MenuItem {
            key: SharedString::from(gen_id()),
            name: String::from("环境"),
            action: Action::Route(RouteAction {
                route: SharedString::from("/environmentList"),
                permission: None,
            }),
        },
        MenuItem {
            key: SharedString::from(gen_id()),
            name: String::from("任务"),
            action: Action::Route(RouteAction {
                route: SharedString::from("/jobList"),
                permission: None,
            }),
        },
        MenuItem {
            key: SharedString::from(gen_id()),
            name: String::from("成员"),
            action: Action::Route(RouteAction {
                route: SharedString::from("/userList"),
                permission: None,
            }),
        },
    ];
    return menus;
}

fn gen_menu_node(
    menu_item: MenuItem,
    permissions: &[String],
    state: Arc<State>,
    navigate: impl Fn(&str, NavigateOptions) + Clone + 'static,
) -> Option<MenuNode> {
    let active_key = state.active_key.clone();
    let mut node = MenuNode {
        state: state.clone(),
        key: menu_item.key.clone(),
        name: menu_item.name,
        action: None,
        children: None,
    };
    match menu_item.action {
        Action::Route(action) => {
            let has_permission = action
                .permission
                .as_ref()
                .map(|permission| permissions.iter().any(|item| return permission == item))
                .unwrap_or(true);
            if has_permission {
                node.action = Some(UnsyncCallback::new(move |_| {
                    navigate(&action.route, Default::default());
                    active_key.set(Some(menu_item.key.clone()));
                }));
                Some(node)
            } else {
                None
            }
        }
        Action::Children(children) => {
            let children = filter_menus(children, permissions, state, navigate);
            if children.is_empty() {
                None
            } else {
                node.children = Some(Arc::new(children));
                Some(node)
            }
        }
    }
}

fn find_active_key(
    menus: &[MenuItem],
    permissions: &[String],
    current_pathname: &str,
) -> Option<Key> {
    for menu_item in menus {
        match &menu_item.action {
            Action::Route(action) => {
                if current_pathname == action.route
                    && action
                        .permission
                        .as_ref()
                        .map(|permission| permissions.iter().any(|item| return permission == item))
                        .unwrap_or(true)
                {
                    return Some(menu_item.key.clone());
                }
            }
            Action::Children(children) => {
                return find_active_key(&children, permissions, current_pathname);
            }
        }
    }
    return None;
}

fn filter_menus(
    menus: Vec<MenuItem>,
    permissions: &[String],
    state: Arc<State>,
    navigate: impl Fn(&str, NavigateOptions) + Clone + 'static,
) -> Vec<MenuNode> {
    let mut filterd_menu_nodes = Vec::with_capacity(menus.len());
    for menu_item in menus {
        if let Some(menu_node) =
            gen_menu_node(menu_item, permissions, state.clone(), navigate.clone())
        {
            filterd_menu_nodes.push(menu_node);
        }
    }
    return filterd_menu_nodes;
}

#[component]
pub fn SysMenu(permissions: Vec<String>) -> impl IntoView {
    let navigate = use_navigate();
    let menus = get_menus();
    let active_key = find_active_key(
        &menus,
        &permissions,
        &web_sys::window().unwrap().location().pathname().unwrap(),
    );
    let expanded_key = RwSignal::new(None);
    let active_key = RwSignal::new(active_key);
    let state = Arc::new(State {
        expanded_key: expanded_key,
        active_key: active_key,
    });
    let list = filter_menus(menus, &permissions, state, navigate);
    view! {
        <Menu list={list.clone()} />
    }
}
