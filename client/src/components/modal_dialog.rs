use super::center_middle::CenterMiddle;
use super::dialog::Dialog;
use super::page::Page;
use crate::LightString;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    title: LightString,
    closable: bool,
    z_index: u64,
    center_style: LightString,
    dialog_style: LightString,
    content_style: LightString,
}

pub enum Msg {
    Close,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub title: LightString,
    pub closable: bool,
    #[prop_or(1)]
    pub z_index: u64,
    #[prop_or(LightString::Static(""))]
    pub center_style: LightString,
    #[prop_or(LightString::Static(""))]
    pub dialog_style: LightString,
    #[prop_or(LightString::Static(""))]
    pub content_style: LightString,
    #[prop_or_default]
    pub onclose: Option<Callback<()>>,
    pub children: Children,
}

pub struct ModalDialog {
    state: State,
    children: Children,
}

impl Component for ModalDialog {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            title: props.title.clone(),
            closable: props.closable,
            z_index: props.z_index,
            center_style: props.center_style.clone(),
            dialog_style: props.dialog_style.clone(),
            content_style: props.content_style.clone(),
        };
        ModalDialog {
            state,
            children: props.children.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Close => match ctx.props().onclose.as_ref() {
                Some(onclose) => {
                    onclose.emit(());
                }
                None => (),
            },
        }
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.title = props.title.clone();
        self.state.closable = props.closable;
        self.state.z_index = props.z_index;
        self.state.center_style = props.center_style.clone();
        self.state.dialog_style = props.dialog_style.clone();
        self.state.content_style = props.content_style.clone();
        self.children = props.children.clone();
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut content_style = String::from("background-color:#FFF;");
        if !self.state.content_style.is_empty() {
            content_style.push_str(&self.state.content_style);
        }
        let on_close = ctx.link().callback(|_| Msg::Close);
        html! {
            <Page mask=true z_index={self.state.z_index}>
                <CenterMiddle content_style={self.state.center_style.clone()}>
                    <Dialog title={self.state.title.clone()} closable={self.state.closable} onclose={Some(on_close)} style={self.state.dialog_style.clone()} content_style={self.state.content_style.clone()}>
                        { self.children.clone() }
                    </Dialog>
                </CenterMiddle>
            </Page>
        }
    }
}
