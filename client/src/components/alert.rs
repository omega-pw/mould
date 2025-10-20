use super::button::Button;
use super::modal_dialog::ModalDialog;
use crate::SharedString;
use js_sys::Function;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    title: SharedString,
    content: SharedString,
    z_index: u64,
    ok_text: SharedString,
}

pub enum Msg {
    Ok,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_else(||SharedString::from("提示"))]
    pub title: SharedString,
    pub content: SharedString,
    #[prop_or(999)]
    pub z_index: u64,
    #[prop_or_else(||SharedString::from("确定"))]
    pub ok_text: SharedString,
    #[prop_or_default]
    pub onok: Option<Callback<()>>,
}

pub struct Alert {
    state: State,
}

impl Component for Alert {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            title: props.title.clone(),
            content: props.content.clone(),
            z_index: props.z_index,
            ok_text: props.ok_text.clone(),
        };
        Alert { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Ok => match ctx.props().onok.as_ref() {
                Some(onok) => {
                    onok.emit(());
                }
                None => (),
            },
        }
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.title = props.title.clone();
        self.state.content = props.content.clone();
        self.state.z_index = props.z_index;
        self.state.ok_text = props.ok_text.clone();
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_mobile = false;
        let center_style = if is_mobile {
            "min-width:40%;max-width:60%;"
        } else {
            "min-width:24em;max-width:48em;"
        };
        let mut footer_style = String::from("text-align:center;");
        if !is_mobile {
            footer_style.push_str("padding-bottom:0.5em;");
        }
        let mut btn_style = String::from("font-size: inherit;");
        if is_mobile {
            btn_style.push_str("display:block;width:100%;background-color:transparent;border:none;border-top-width:1px;border-top-style:solid;border-top-color:#EEEEEE");
        }
        let on_click = ctx.link().callback(|_| Msg::Ok);
        html! {
            <ModalDialog title={self.state.title.clone()} closable=false z_index={self.state.z_index} center_style={center_style}>
                <div style="min-height: 2em;padding:0.5em;">
                    {&self.state.content}
                </div>
                <div style={footer_style}>
                    <Button onclick={on_click} style={btn_style}>{self.state.ok_text.clone()}</Button>
                </div>
            </ModalDialog>
        }
    }
}

pub fn alert(content: SharedString, title: Option<SharedString>, cb: Option<impl Fn() + 'static>) {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let alert_root = document.create_element("div").unwrap();
    let on_root_click: Function = Closure::wrap(Box::new(|event: Event| {
        //阻止mousedown事件冒泡，防止点击事件被document补货到，导致FocusArea组件触发离开事件
        event.stop_propagation();
    }) as Box<dyn FnMut(Event)>)
    .into_js_value()
    .dyn_into()
    .unwrap();
    alert_root
        .add_event_listener_with_callback("mousedown", &on_root_click)
        .unwrap();
    body.append_child(&alert_root).unwrap();
    let alert_handle: Rc<Cell<Option<AppHandle<Alert>>>> = Rc::new(Cell::new(None));
    let alert_handle_clone = alert_handle.clone();
    let alert_root_clone = alert_root.clone();
    let props = Props {
        title: title.unwrap_or_else(|| SharedString::from("提示")),
        content: content,
        z_index: 999,
        ok_text: SharedString::from("确定"),
        onok: Some(Callback::from(move |_: ()| {
            if let Some(cb) = cb.as_ref() {
                cb();
            }
            if let Some(alert_handle) = alert_handle_clone.take() {
                alert_handle.destroy();
                let document = web_sys::window().unwrap().document().unwrap();
                let body = document.body().unwrap();
                body.remove_child(&alert_root_clone).unwrap();
            }
            alert_root_clone
                .remove_event_listener_with_callback("mousedown", &on_root_click)
                .unwrap();
        })),
    };
    alert_handle.set(Some(
        yew::Renderer::<Alert>::with_root_and_props(alert_root, props).render(),
    ));
}
