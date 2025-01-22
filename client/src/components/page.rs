use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    mask: bool,
    z_index: u64,
}

pub enum Msg {}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub mask: bool,
    #[prop_or(1)]
    pub z_index: u64,
    pub children: Children,
}

pub struct Page {
    state: State,
    children: Children,
}

impl Component for Page {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            mask: props.mask,
            z_index: props.z_index,
        };
        Page {
            state,
            children: props.children.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.mask = props.mask;
        self.state.z_index = props.z_index;
        self.children = props.children.clone();
        return true;
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut style = format!("position: absolute;top: 0;left: 0;bottom: 0;right: 0;width: 100%;height: 100%;overflow: hidden;z-index: {};", self.state.z_index);
        if self.state.mask {
            style.push_str("background-color:rgba(128,128,128,0.5);");
        }
        html! {
            <div style={style}>
                { self.children.clone() }
            </div>
        }
    }
}
