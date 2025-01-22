use crate::LightString;
use yew::html::Scope;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    title: LightString,
    closable: bool,
    style: LightString,
    content_style: LightString,
}

pub enum Msg {
    Close,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub title: LightString,
    pub closable: bool,
    #[prop_or_default]
    pub style: LightString,
    #[prop_or_default]
    pub content_style: LightString,
    #[prop_or_default]
    pub onclose: Option<Callback<()>>,
    pub children: Children,
}

pub struct Dialog {
    state: State,
    children: Children,
}

impl Component for Dialog {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            title: props.title.clone(),
            closable: props.closable,
            style: props.style.clone(),
            content_style: props.content_style.clone(),
        };
        Dialog {
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
        self.state.closable = props.closable.clone();
        self.state.style = props.style.clone();
        self.state.content_style = props.content_style.clone();
        self.children = props.children.clone();
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut content_style = String::from("background-color:#FFF;");
        if !self.state.content_style.is_empty() {
            content_style.push_str(&self.state.content_style);
        }
        html! {
            <div style={self.state.style.clone()}>
                <div class="e-title-bar" style="height: 2em;line-height: 2em;position: relative;font-weight: normal;margin: 0;padding-left: 0.5em;">
                    {&self.state.title}
                    {self.close_view(ctx.link())}
                </div>
                <div style={content_style}>
                    { self.children.clone() }
                </div>
            </div>
        }
    }
}

impl Dialog {
    fn close_view(&self, link: &Scope<Self>) -> Html {
        if self.state.closable {
            let on_close = link.callback(|_| Msg::Close);
            html! {
                <span class="btn-close" onclick={on_close} style="position: absolute;top: 0;right: 0;width: 2em;height: 2em;text-align: center;cursor: pointer;">
                    <i class="fas fa-times"></i>
                </span>
            }
        } else {
            html! {}
        }
    }
}
