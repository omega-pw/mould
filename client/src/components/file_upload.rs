use super::button::Button;
use super::hidden_file::HiddenFile;
use super::File;
use super::HashingFile;
use std::ops::Deref;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::{html, Component, Context, Html};

pub enum Msg {
    ReplaceFile(web_sys::File),
    Remove,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub file: Option<File>,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub onchange: Option<Callback<Option<File>>>,
}

pub struct FileUpload {}

impl Component for FileUpload {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        FileUpload {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let calc_file_sha512_method: js_sys::Function = js_sys::Reflect::get(
            &web_sys::window().unwrap(),
            &JsValue::from_str("calcFileSha512"),
        )
        .unwrap()
        .dyn_into()
        .unwrap();
        match msg {
            Msg::ReplaceFile(file) => {
                if let Some(onchange) = ctx.props().onchange.as_ref() {
                    let file = File::Local(HashingFile {
                        file: file.clone(),
                        sha512: calc_file_sha512_method
                            .call1(&wasm_bindgen::JsValue::UNDEFINED, &file)
                            .unwrap()
                            .dyn_into()
                            .unwrap(),
                    });
                    onchange.emit(Some(file));
                }
            }
            Msg::Remove => {
                if let Some(onchange) = ctx.props().onchange.as_ref() {
                    onchange.emit(None);
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
        let file_view = props.file.as_ref().map(|file| self.file_view(file));
        html! {
            <div style="overflow: hidden;">
                {
                    if props.readonly {
                        html! {
                            <div style="display: inline-block;">
                                {file_view}
                            </div>
                        }
                    } else {
                        let on_replace_file = ctx.link().callback(move |mut files: Vec<web_sys::File>| {
                            Msg::ReplaceFile(files.pop().unwrap())
                        });
                        if let Some(file_view) = file_view {
                            let on_remove = ctx.link().callback(move |_| Msg::Remove);
                            html! {
                                <div style="display: inline-block;">
                                    <HiddenFile onfiles={on_replace_file} root_style="display: inline-block;">
                                        { file_view }
                                    </HiddenFile>
                                    <Button onclick={on_remove} style="margin-left: 0.25em;">{"删除"}</Button>
                                </div>
                            }
                        } else {
                            html! {
                                <HiddenFile onfiles={on_replace_file}>
                                    <Button>{"添加"}</Button>
                                </HiddenFile>
                            }
                        }
                    }
                }
            </div>
        }
    }
}

impl FileUpload {
    fn file_view(&self, file: &File) -> Html {
        match file {
            File::Remote { key, name, .. } => {
                let url = format!("/blob/{}", key);
                html! {
                    <a href={url} target="_blank">{name}</a>
                }
            }
            File::Local(hashing_file) => {
                html! { hashing_file.file.name() }
            }
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps {
    pub file: UseStateHandle<Option<File>>,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub onchange: Option<Callback<Option<File>>>,
}

#[function_component]
pub fn BindingFileUpload(props: &BindingProps) -> Html {
    let file_clone = props.file.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |file: Option<File>| {
        file_clone.set(file.clone());
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(file);
        }
    });
    return html! {
        <FileUpload
            file={props.file.deref().clone()}
            readonly={props.readonly}
            onchange={on_change}
        />
    };
}
