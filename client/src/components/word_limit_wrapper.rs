use crate::LightString;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    length: u32,
    maxlength: u32,
    style: LightString,
    limit_style: LightString,
}

pub enum Msg {}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub length: u32,
    pub maxlength: u32,
    #[prop_or(LightString::Static(""))]
    pub style: LightString,
    #[prop_or(LightString::Static(""))]
    pub limit_style: LightString,
    pub children: Children,
}

pub struct WordLimitWrapper {
    state: State,
    children: Children,
}

impl Component for WordLimitWrapper {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            length: props.length,
            maxlength: props.maxlength,
            style: props.style.clone(),
            limit_style: props.limit_style.clone(),
        };
        WordLimitWrapper {
            state,
            children: props.children.clone(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.length = props.length;
        self.state.maxlength = props.maxlength;
        self.state.style = props.style.clone();
        self.state.limit_style = props.limit_style.clone();
        self.children = props.children.clone();
        return true;
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        return true;
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut style = String::from("position: relative;display: inline-block;");
        if !self.state.style.is_empty() {
            style.push_str(&self.state.style);
        }
        let limit_words = format!("{}/{}", self.state.length, self.state.maxlength);
        let mut limit_style = String::from("margin: 0;position: absolute;line-height: 1.25em;bottom: 0.25em;right: 0.5em;text-align: right;");
        if !self.state.limit_style.is_empty() {
            limit_style.push_str(&self.state.limit_style);
        }
        html! {
            <div style={style}>
                { self.children.clone() }
                <p style={limit_style}>{limit_words}</p>
            </div>
        }
    }
}
