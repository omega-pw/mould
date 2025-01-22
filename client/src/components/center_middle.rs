use crate::LightString;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

struct State {
    content_style: LightString,
}

pub enum Msg {}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or(LightString::Static(""))]
    pub content_style: LightString,
    pub children: Children,
}

pub struct CenterMiddle {
    state: State,
    children: Children,
}

impl Component for CenterMiddle {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = State {
            content_style: props.content_style.clone(),
        };
        CenterMiddle {
            state,
            children: props.children.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        return true;
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.state.content_style = props.content_style.clone();
        self.children = props.children.clone();
        return true;
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut style = String::from("display:inline-block;text-align: initial;");
        if !self.state.content_style.is_empty() {
            style.push_str(&self.state.content_style);
        }
        html! {
            <table style="width:100%;height: 100%;border-collapse: collapse;table-layout: auto;border: 0;">
                <tbody style="width:100%;height: 100%;">
                    <tr style="width:100%;height: 100%;">
                        <td style="width:100%;height: 100%;vertical-align:middle;text-align: center;overflow: hidden;">
                            //text-align: initial 还原成默认的值
                            <div style={style}>
                                { self.children.clone() }
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        }
    }
}
