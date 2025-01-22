use super::center_middle::CenterMiddle;
use super::dialog::Dialog;
use super::page::Page;
use super::uploading_file::UploadingFile;
use super::HashingFile;
use crate::utils::gen_id;
use crate::LightString;
use js_sys::Function;
use js_sys::Promise;
use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::{html, Html};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub files: Vec<(Key, HashingFile, Callback<Result<String, LightString>>)>,
    #[prop_or(999)]
    pub z_index: u64,
    #[prop_or_default]
    pub ondone: Callback<Result<(), LightString>>,
}

#[function_component]
pub fn UploadingFiles(props: &Props) -> Html {
    let first_error_result: UseStateHandle<Result<(), LightString>> = use_state(|| Ok(()));
    let done_count: UseStateHandle<AtomicUsize> = use_state(Default::default);
    let count = props.files.len();
    html! {
        <Page mask=true z_index={props.z_index}>
            <CenterMiddle>
                <Dialog title="正在上传" closable={false} style="width: 24em;" content_style="max-height: 24em;padding: 0.5em;overflow:auto;">
                    {
                        for props.files.iter().map(|(key, hashing_file, onsingledone)| {
                            let ondone = props.ondone.clone();
                            let done_count = done_count.clone();
                            let onsingledone = onsingledone.clone();
                            let first_error_result = first_error_result.clone();
                            let ondone = Callback::from(move |result: Result<String, LightString>| {
                                let has_error = first_error_result.is_err();
                                if let Err(error) = result.as_ref() {
                                    if !has_error {
                                        first_error_result.set(Err(error.clone()));
                                    }
                                }
                                onsingledone.emit(result.clone());
                                if count == 1 + done_count.fetch_add(1, Ordering::Relaxed) {
                                    if has_error {
                                        ondone.emit(first_error_result.deref().clone());
                                    } else {
                                        ondone.emit(result.map(|_| ()));
                                    }
                                }
                            });
                            html! {
                                <UploadingFile key={key.clone()} file={hashing_file.clone()} ondone={ondone}/>
                            }
                        })
                    }
                </Dialog>
            </CenterMiddle>
        </Page>
    }
}

pub async fn upload_files(
    files: Vec<(HashingFile, Callback<Result<String, LightString>>)>,
) -> Result<(), LightString> {
    if files.is_empty() {
        return Ok(());
    }
    let mut files = Some(files);
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let root = document.create_element("div").unwrap();
    let root_clone = root.clone();
    let handle: Rc<Cell<Option<AppHandle<UploadingFiles>>>> = Rc::new(Cell::new(None));
    let handle_clone = handle.clone();
    let mut promise_fn = move |resolve: Function, reject: Function| {
        body.append_child(&root).unwrap();
        let props = Props {
            files: files
                .take()
                .unwrap_or_default()
                .into_iter()
                .map(|(hashing_file, ondone)| (gen_id().into(), hashing_file, ondone))
                .collect(),
            z_index: 999,
            ondone: Callback::from(move |result: Result<(), LightString>| match result {
                Ok(_) => {
                    if let Err(err) = resolve.call0(&wasm_bindgen::JsValue::UNDEFINED) {
                        log::error!("调用Promise的resolve失败: {:?}", err);
                    }
                }
                Err(error) => {
                    if let Err(err) = reject.call1(
                        &wasm_bindgen::JsValue::UNDEFINED,
                        &wasm_bindgen::JsValue::from_str(&error),
                    ) {
                        log::error!("调用Promise的reject失败: {:?}", err);
                    }
                }
            }),
        };
        handle.set(Some(
            yew::Renderer::<UploadingFiles>::with_root_and_props(root.clone(), props).render(),
        ));
    };
    let promise = Promise::new(&mut promise_fn);
    let result = JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|err| -> LightString {
            log::error!("上传文件失败: {:?}", err);
            return "上传文件失败".into();
        });
    if let Some(handle) = handle_clone.take() {
        handle.destroy();
        let body = document.body().unwrap();
        body.remove_child(&root_clone).unwrap();
    }
    return result;
}
