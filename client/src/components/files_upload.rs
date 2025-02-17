use super::hidden_file::HiddenFile;
use super::File;
use super::HashingFile;
use crate::utils::gen_id;
use std::marker::PhantomData;
use std::ops::Deref;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::{html, Component, Context, Html};

pub enum Msg {
    AppendFiles(Vec<web_sys::File>),
    ReplaceFiles(Vec<web_sys::File>, usize),
    Remove(usize),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<O: Default + Clone + PartialEq> {
    pub files: Vec<(Key, File, O)>,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub onchange: Option<Callback<Vec<(Key, File, O)>>>,
}

pub struct FilesUpload<O> {
    phantom: PhantomData<O>,
}

impl<O: Default + Clone + PartialEq + 'static> Component for FilesUpload<O> {
    type Message = Msg;
    type Properties = Props<O>;

    fn create(_ctx: &Context<Self>) -> Self {
        FilesUpload {
            phantom: PhantomData,
        }
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
            Msg::AppendFiles(files) => {
                if let (Some(onchange), Some(file)) = (ctx.props().onchange.as_ref(), files.first())
                {
                    let mut files = ctx.props().files.clone();
                    files.push((
                        gen_id().into(),
                        File::Local(HashingFile {
                            file: file.clone(),
                            sha512: calc_file_sha512_method
                                .call1(&wasm_bindgen::JsValue::UNDEFINED, file)
                                .unwrap()
                                .dyn_into()
                                .unwrap(),
                        }),
                        Default::default(),
                    ));
                    onchange.emit(files);
                }
            }
            Msg::ReplaceFiles(files, index) => {
                if let (Some(onchange), Some(file)) = (ctx.props().onchange.as_ref(), files.first())
                {
                    let mut files = ctx.props().files.clone();
                    let file = File::Local(HashingFile {
                        file: file.clone(),
                        sha512: calc_file_sha512_method
                            .call1(&wasm_bindgen::JsValue::UNDEFINED, file)
                            .unwrap()
                            .dyn_into()
                            .unwrap(),
                    });
                    files[index].1 = file;
                    onchange.emit(files);
                }
            }
            Msg::Remove(index) => {
                if let Some(onchange) = ctx.props().onchange.as_ref() {
                    let mut files = ctx.props().files.clone();
                    files.remove(index);
                    onchange.emit(files);
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
        let on_append_files = ctx.link().callback(Msg::AppendFiles);
        html! {
            <div style="overflow: hidden;">
                {
                    for props.files.iter().enumerate().map(|(index, (key, file, _))| {
                        let file_view = self.file_view(file);
                        {
                            if props.readonly {
                                html! {
                                    <div key={key.clone()}>
                                        {file_view}
                                    </div>
                                }
                            } else {
                                let on_replace_files = ctx.link().callback(move |files| {
                                    Msg::ReplaceFiles(files, index)
                                });
                                let on_remove = ctx.link().callback(move |_| {
                                    Msg::Remove(index)
                                });
                                html! {
                                    <div key={key.clone()}>
                                        <HiddenFile onfiles={on_replace_files} root_style="display:inline-block;">
                                            {file_view}
                                        </HiddenFile>
                                        <button type="button" class="e-btn" onclick={on_remove} style="margin-left:0.25em;">{"删除"}</button>
                                    </div>
                                }
                            }
                        }
                    })
                }
                {
                    if props.readonly {
                        html! {}
                    } else {
                        html! {
                            <HiddenFile onfiles={on_append_files}>
                                <button type="button" class="e-btn">{"添加"}</button>
                            </HiddenFile>
                        }
                    }
                }
            </div>
        }
    }
}

impl<O: Default> FilesUpload<O> {
    fn file_view(&self, file: &File) -> Html {
        match file {
            File::Remote { key, name, .. } => {
                let url = format!("/{}", key);
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
pub struct BindingProps<O: Default + Clone + PartialEq> {
    pub files: UseStateHandle<Vec<(Key, File, O)>>,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub onchange: Option<Callback<Vec<(Key, File, O)>>>,
}

#[function_component]
pub fn BindingFilesUpload<O: Default + Clone + PartialEq + 'static>(
    props: &BindingProps<O>,
) -> Html {
    let files_clone = props.files.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |files: Vec<(Key, File, O)>| {
        files_clone.set(files.clone());
        if let Some(onchange) = onchange.as_ref() {
            onchange.emit(files);
        }
    });
    return html! {
        <FilesUpload<O>
            files={props.files.deref().clone()}
            readonly={props.readonly}
            onchange={on_change}
        />
    };
}
