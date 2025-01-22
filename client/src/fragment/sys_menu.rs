use crate::components::menu::Menu;
use crate::components::menu::MenuNode;
use crate::components::menu::State;
use crate::route::Route;
use crate::utils::gen_id;
use std::sync::Arc;
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::AnyRoute;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub route: Route,
}

#[function_component]
pub fn SysMenu(props: &Props) -> Html {
    let expanded_key = use_state(|| None);
    let route_state = use_state(|| None);
    let route_state_clone = route_state.clone();
    let route = props.route.to_path();
    use_effect_with(props.route.clone(), move |_| {
        route_state_clone.set(Some(route));
        || ()
    });
    let state = Arc::new(State {
        expanded_key: expanded_key,
        route: route_state,
    });
    let list = vec![
        MenuNode {
            state: state.clone(),
            key: gen_id().into(),
            name: String::from("环境规格"),
            route: Some(AnyRoute::new(Route::EnvironmentSchemaList.to_path())),
            children: None,
        },
        MenuNode {
            state: state.clone(),
            key: gen_id().into(),
            name: String::from("环境"),
            route: Some(AnyRoute::new(Route::EnvironmentList.to_path())),
            children: None,
        },
        MenuNode {
            state: state.clone(),
            key: gen_id().into(),
            name: String::from("任务"),
            route: Some(AnyRoute::new(Route::JobList.to_path())),
            children: None,
        },
        MenuNode {
            state: state.clone(),
            key: gen_id().into(),
            name: String::from("成员"),
            route: Some(AnyRoute::new(Route::UserList.to_path())),
            children: None,
        },
    ];
    html! {
        <Menu list={list.clone()} />
    }
}
