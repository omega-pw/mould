use super::button::Button;
use super::hidden_file::HiddenFile;
use super::HashingFile;
use super::Resource;
use crate::SharedString;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[component]
pub fn FileUpload(
    file: RwSignal<Option<Resource>, LocalStorage>,
    #[prop(optional)] readonly: bool,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<Option<Resource>>>,
) -> impl IntoView {
    let file_view = {
        let file = file.clone();
        move || file.read().as_ref().map(file_view)
    };
    view! {
        <div style="overflow: hidden;">
            {
                if readonly {
                    view! {
                        <div style="display: inline-block;">
                            {file_view}
                        </div>
                    }.into_any()
                } else {
                    let on_replace_file = {
                        let file = file.clone();
                        UnsyncCallback::new(move |files: Vec<web_sys::File>| {
                            if let Some(new_file) = files.first() {
                                let calc_file_sha512_method: js_sys::Function = js_sys::Reflect::get(
                                    &web_sys::window().unwrap(),
                                    &JsValue::from_str("calcFileSha512"),
                                )
                                .unwrap()
                                .dyn_into()
                                .unwrap();
                                let new_file = Resource::Local(HashingFile {
                                    file: new_file.clone(),
                                    sha512: calc_file_sha512_method
                                        .call1(&wasm_bindgen::JsValue::UNDEFINED, &new_file)
                                        .unwrap()
                                        .dyn_into()
                                        .unwrap(),
                                });
                                if !readonly {
                                    file.set(Some(new_file.clone()));
                                }
                                if let Some(onchange) = onchange.as_ref() {
                                    onchange.run(Some(new_file));
                                }
                            }
                        })
                    };
                    (move || {
                        if let Some(file_view) = file_view() {
                            let on_remove = UnsyncCallback::new(move |_| {
                                if !readonly {
                                    file.set(None);
                                }
                                if let Some(onchange) = onchange.as_ref() {
                                    onchange.run(None);
                                }
                            });
                            view! {
                                <div style="display: inline-block;">
                                    <HiddenFile onfiles={on_replace_file} root_style="display: inline-block;">
                                        { file_view }
                                    </HiddenFile>
                                    <Button onclick={on_remove} style=SharedString::from("margin-left: 0.25em;")>{"删除"}</Button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <HiddenFile onfiles={on_replace_file}>
                                    <Button>{"添加"}</Button>
                                </HiddenFile>
                            }.into_any()
                        }
                    }).into_any()
                }
            }
        </div>
    }
}

fn file_view(file: &Resource) -> AnyView {
    match file {
        Resource::Remote(metadata) => {
            let url = format!("/{}", metadata.key);
            view! {
                <a href={url} target="_blank">{metadata.name.clone()}</a>
            }
            .into_any()
        }
        Resource::Local(hashing_file) => hashing_file.file.name().into_any(),
    }
}
