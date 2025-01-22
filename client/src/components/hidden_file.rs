use wasm_bindgen::JsCast;
use web_sys::File;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

pub enum Msg {
    Files(Vec<File>),
    Click,
    Noop,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub root_style: Option<String>,
    #[prop_or_default]
    pub style: Option<String>,
    #[prop_or_default]
    pub accept: Option<String>,
    #[prop_or_default]
    pub onfiles: Option<Callback<Vec<File>>>,
    pub children: Children,
}

pub struct HiddenFile {
    file_ref: NodeRef,
}

impl Component for HiddenFile {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        HiddenFile {
            file_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(files) => {
                if let Some(onfiles) = ctx.props().onfiles.as_ref() {
                    onfiles.emit(files);
                }
            }
            Msg::Click => {
                let file_ref = &self.file_ref;
                if let Some(input_dom) = file_ref.cast::<HtmlInputElement>() {
                    input_dom.click();
                }
            }
            Msg::Noop => (),
        }
        return true;
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        return true;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_file_change = ctx.link().callback(|evt: Event| {
            if let Some(target) = evt.target() {
                match target.dyn_into::<HtmlInputElement>() {
                    Ok(input_dom) => {
                        if let Some(files) = input_dom.files() {
                            let len = files.length();
                            let files: Vec<File> = (0..len)
                                .into_iter()
                                .map(|index| {
                                    return files.get(index).unwrap();
                                })
                                .collect();
                            input_dom.set_value("");
                            Msg::Files(files)
                        } else {
                            Msg::Noop
                        }
                    }
                    Err(err) => {
                        log::error!("{:?}", err);
                        Msg::Noop
                    }
                }
            } else {
                Msg::Noop
            }
        });
        let on_click = ctx.link().callback(move |_| Msg::Click);
        html! {
            <div style={ctx.props().root_style.clone()}>
                <input ref={&self.file_ref} type="file" onchange={on_file_change} accept={ctx.props().accept.clone()} style="display:none;"/>
                <div onclick={on_click} style={ctx.props().style.clone()}>
                  { ctx.props().children.clone() }
                </div>
            </div>
        }
    }
}
