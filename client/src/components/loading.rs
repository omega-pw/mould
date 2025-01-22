use web_sys::Element;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    center_middle: bool,
    z_index: u64,
}

pub enum Msg {}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub center_middle: bool,
    #[prop_or(999)]
    pub z_index: u64,
}

pub struct Loading {
    state: State,
}

impl Component for Loading {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            center_middle: props.center_middle,
            z_index: props.z_index,
        };
        Loading { state }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.center_middle = props.center_middle;
        self.state.z_index = props.z_index;
        return true;
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if self.state.center_middle {
            let style = format!("width: 3em;height: 3em;position: fixed;left: 50%;top: 50%;-ms-transform: translateX(-50%) translateY(-50%);-moz-transform: translateX(-50%) translateY(-50%);-webkit-transform: translateX(-50%) translateY(-50%);-o-transform: translateX(-50%) translateY(-50%);transform: translateX(-50%) translateY(-50%);z-index: {};",self.state.z_index);
            html! {
                <div style={style}>
                    <i class="loading" style="width: 100%;height: 100%;"></i>
                </div>
            }
        } else {
            html! {
                <i class="loading" style="width: 3em;height: 3em;"></i>
            }
        }
    }
}

static mut INSTANCE: Option<(AppHandle<Loading>, Element, u32)> = None;

pub fn show() {
    let inst = unsafe { INSTANCE.as_mut() };
    if let Some(inst) = inst {
        inst.2 += 1;
        return;
    }
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let loading_root = document.create_element("div").unwrap();
    body.append_child(&loading_root).unwrap();
    let props = Props {
        center_middle: true,
        z_index: 999,
    };
    let loading_handle =
        yew::Renderer::<Loading>::with_root_and_props(loading_root.clone(), props).render();
    unsafe {
        INSTANCE.replace((loading_handle, loading_root, 1));
    }
}

fn destroy() {
    let inst = unsafe { INSTANCE.take() };
    if let Some((loading_handle, loading_root, _)) = inst {
        loading_handle.destroy();
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        body.remove_child(&loading_root).unwrap();
    }
}

pub fn hide() {
    let inst = unsafe { INSTANCE.as_mut() };
    if let Some(inst) = inst {
        if 1 >= inst.2 {
            destroy();
        } else {
            inst.2 -= 1;
        }
    }
}
