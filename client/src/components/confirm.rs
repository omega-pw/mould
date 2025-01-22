use super::modal_dialog::ModalDialog;
use crate::LightString;
use std::cell::Cell;
use std::rc::Rc;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    title: LightString,
    content: LightString,
    z_index: u64,
    ok_text: LightString,
    cancel_text: LightString,
}

pub enum Msg {
    Done(bool),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_else(||LightString::from("确认"))]
    pub title: LightString,
    pub content: LightString,
    #[prop_or(999)]
    pub z_index: u64,
    #[prop_or_else(||LightString::from("确定"))]
    pub ok_text: LightString,
    #[prop_or_else(||LightString::from("取消"))]
    pub cancel_text: LightString,
    pub ondone: Callback<bool>,
}

pub struct Confirm {
    state: State,
}

impl Component for Confirm {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            title: props.title.clone(),
            content: props.content.clone(),
            z_index: props.z_index,
            ok_text: props.ok_text.clone(),
            cancel_text: props.cancel_text.clone(),
        };
        Confirm { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Done(ok) => {
                ctx.props().ondone.emit(ok);
            }
        }
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.title = props.title.clone();
        self.state.content = props.content.clone();
        self.state.z_index = props.z_index;
        self.state.ok_text = props.ok_text.clone();
        self.state.cancel_text = props.cancel_text.clone();
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
        let on_ok = ctx.link().callback(|_| Msg::Done(true));
        let on_cancel = ctx.link().callback(|_| Msg::Done(false));
        html! {
            <ModalDialog title={self.state.title.clone()} closable=false z_index={self.state.z_index} center_style={center_style}>
                <div style="min-height: 2em;padding:0.5em;">{&self.state.content}</div>
                <div style={footer_style}>
                    <button type="button" class="e-btn" onclick={on_ok} style={btn_style.clone()}>{&self.state.ok_text}</button>
                    <button type="button" class="e-btn" onclick={on_cancel} style={btn_style.clone()}>{&self.state.cancel_text}</button>
                </div>
            </ModalDialog>
        }
    }
}

pub fn confirm(content: LightString, title: Option<LightString>, cb: impl Fn(bool) + 'static) {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let confirm_root = document.create_element("div").unwrap();
    body.append_child(&confirm_root).unwrap();
    let confirm_handle: Rc<Cell<Option<AppHandle<Confirm>>>> = Rc::new(Cell::new(None));
    let confirm_handle_clone = confirm_handle.clone();
    let confirm_root_clone = confirm_root.clone();
    let props = Props {
        title: title.unwrap_or_else(|| LightString::from("确认")),
        content: content,
        z_index: 999,
        ok_text: LightString::from("确定"),
        cancel_text: LightString::from("取消"),
        ondone: Callback::from(move |ret: bool| {
            cb(ret);
            if let Some(confirm_handle) = confirm_handle_clone.take() {
                confirm_handle.destroy();
                let document = web_sys::window().unwrap().document().unwrap();
                let body = document.body().unwrap();
                body.remove_child(&confirm_root_clone).unwrap();
            }
        }),
    };
    confirm_handle.set(Some(
        yew::Renderer::<Confirm>::with_root_and_props(confirm_root, props).render(),
    ));
}
