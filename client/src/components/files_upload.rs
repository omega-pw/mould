use super::hidden_file::HiddenFile;
use super::HashingFile;
use super::Resource;
use crate::utils::gen_id;
use crate::Key;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::File;

#[component]
pub fn FilesUpload<O>(
    files: RwSignal<Vec<(Key, RwSignal<Resource, LocalStorage>, O)>, LocalStorage>,
    #[prop(optional)] readonly: bool,
    #[prop(into, default = None)] onchange: Option<
        UnsyncCallback<Vec<(Key, RwSignal<Resource, LocalStorage>, O)>>,
    >,
) -> impl IntoView
where
    O: Default + Clone + Send + 'static,
{
    let on_append_files = {
        let files = files.clone();
        let onchange = onchange.clone();
        UnsyncCallback::new(move |new_files: Vec<File>| {
            if readonly {
                return;
            }
            if let (Some(onchange), Some(file)) = (onchange.as_ref(), new_files.first()) {
                let calc_file_sha512_method: js_sys::Function = js_sys::Reflect::get(
                    &web_sys::window().unwrap(),
                    &JsValue::from_str("calcFileSha512"),
                )
                .unwrap()
                .dyn_into()
                .unwrap();
                files.write().push((
                    gen_id().into(),
                    RwSignal::new_local(Resource::Local(HashingFile {
                        file: file.clone(),
                        sha512: calc_file_sha512_method
                            .call1(&wasm_bindgen::JsValue::UNDEFINED, file)
                            .unwrap()
                            .dyn_into()
                            .unwrap(),
                    })),
                    Default::default(),
                ));
                onchange.run(files.get());
            }
        })
    };
    view! {
        <div style="overflow: hidden;">
            <For
                each={
                    let files = files.clone();
                    move || { files.get().into_iter().enumerate() }
                }
                key=|(_index, (key, _file, _))| { key.clone() }
                children=move |(index, (_key, file, _))| {
                    let file_view = {
                        let file = file.clone();
                        move || file_view(&file.read())
                    };
                    {
                        if readonly {
                            view! {
                                <div>
                                    {file_view}
                                </div>
                            }.into_any()
                        } else {
                            let on_replace_files = {
                                let files = files.clone();
                                let onchange = onchange.clone();
                                UnsyncCallback::new(move |new_files: Vec<File>| {
                                    if readonly {
                                        return;
                                    }
                                    if let (Some(onchange), Some(new_file)) = (onchange.as_ref(), new_files.first()) {
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
                                                .call1(&wasm_bindgen::JsValue::UNDEFINED, new_file)
                                                .unwrap()
                                                .dyn_into()
                                                .unwrap(),
                                        });
                                        file.set(new_file);
                                        onchange.run(files.get());
                                    }
                                })
                            };
                            let on_remove = move |_| {
                                if readonly {
                                    return;
                                }
                                files.write().remove(index);
                                if let Some(onchange) = onchange.as_ref() {
                                    onchange.run(files.get());
                                }
                            };
                            view! {
                                <div>
                                    <HiddenFile onfiles={on_replace_files} root_style="display:inline-block;">
                                        {file_view}
                                    </HiddenFile>
                                    <button type="button" class="e-btn" on:click={on_remove} style="margin-left:0.25em;">{"删除"}</button>
                                </div>
                            }.into_any()
                        }
                    }
                }
            />
            <Show when=move || { readonly }>
                <HiddenFile onfiles={on_append_files}>
                    <button type="button" class="e-btn">{"添加"}</button>
                </HiddenFile>
            </Show>
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
