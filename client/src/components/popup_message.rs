use super::common_popup::CommonPopup;
use gloo::timers::callback::Timeout;
use web_sys::Element;
use yew::prelude::*;
use yew::AttrValue;
use yew::{html, Html};

#[derive(Clone, PartialEq)]
pub enum MsgType {
    Success,
    Warning,
    Error,
}

pub enum Msg {
    Show,
    Hide,
    Done,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub msg_type: MsgType,
    pub message: AttrValue,
    pub z_index: Option<i32>,
    pub duration: Option<u32>,
    pub ondone: Option<Callback<()>>,
}

pub struct PopupMessage {
    pub active: bool,
    pub timeout: Option<Timeout>,
}

impl Component for PopupMessage {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        PopupMessage {
            active: false,
            timeout: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let link = ctx.link().clone();
            self.timeout.replace(Timeout::new(0, move || {
                link.send_message(Msg::Show);
            }));
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Show => {
                self.active = true;
                let link = ctx.link().clone();
                let duration = ctx.props().duration.unwrap_or(3000);
                self.timeout.replace(Timeout::new(duration, move || {
                    link.send_message(Msg::Hide);
                }));
            }
            Msg::Hide => {
                self.active = false;
                let link = ctx.link().clone();
                self.timeout.replace(Timeout::new(500, move || {
                    link.send_message(Msg::Done);
                }));
            }
            Msg::Done => {
                if let Some(timeout) = self.timeout.take() {
                    timeout.cancel();
                }
                if let Some(ondone) = ctx.props().ondone.as_ref() {
                    ondone.emit(());
                }
            }
        }
        return true;
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let mut style = None;
        if let Some(z_index) = props.z_index {
            style.replace(format!("z-index: {}", z_index));
        }
        let class_suffix = match props.msg_type {
            MsgType::Success => "success",
            MsgType::Warning => "warning",
            MsgType::Error => "error",
        };
        html! {
            <CommonPopup active={self.active} z_index={props.z_index}>
                <div class={format!("msg-{}", class_suffix)}>{ props.message.clone() }</div>
            </CommonPopup>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some(timeout) = self.timeout.take() {
            timeout.cancel();
        }
    }
}

static mut INSTANCE: Option<(AppHandle<PopupMessage>, Element)> = None;

fn destroy() {
    let inst = unsafe { INSTANCE.take() };
    if let Some((popup_message_handle, popup_message_root)) = inst {
        popup_message_handle.destroy();
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        body.remove_child(&popup_message_root).unwrap();
    }
}

pub fn show(msg_type: MsgType, message: AttrValue, duration: Option<u32>, z_index: Option<i32>) {
    destroy();
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let popup_message_root = document.create_element("div").unwrap();
    body.append_child(&popup_message_root).unwrap();
    let ondone = Callback::from(move |_: ()| {
        destroy();
    });
    let props = Props {
        msg_type: msg_type,
        message: message,
        z_index: Some(z_index.unwrap_or(999)),
        duration: duration,
        ondone: Some(ondone),
    };
    let popup_message_handle =
        yew::Renderer::<PopupMessage>::with_root_and_props(popup_message_root.clone(), props)
            .render();
    unsafe {
        INSTANCE.replace((popup_message_handle, popup_message_root));
    }
}

pub fn success(message: AttrValue) {
    show(MsgType::Success, message, None, None);
}

pub fn warning(message: AttrValue) {
    show(MsgType::Warning, message, None, None);
}

pub fn error(message: AttrValue) {
    show(MsgType::Error, message, None, None);
}
