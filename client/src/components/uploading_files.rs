use super::center_middle::CenterMiddle;
use super::dialog::Dialog;
use super::page::Page;
use super::uploading_file::UploadingFile;
use super::HashingFile;
use super::ResourceMetadata;
use crate::utils::gen_id;
use crate::Key;
use crate::SharedString;
use js_sys::Function;
use js_sys::Promise;
use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyViewState;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn UploadingFiles(
    files: Vec<(
        Key,
        Signal<HashingFile, LocalStorage>,
        UnsyncCallback<Result<ResourceMetadata, SharedString>>,
    )>,
    #[prop(default = 999)] z_index: u64,
    ondone: UnsyncCallback<Result<(), SharedString>>,
) -> impl IntoView {
    let first_error_result: RwSignal<Result<(), SharedString>> = RwSignal::new(Ok(()));
    let done_count: Arc<AtomicUsize> = Arc::new(Default::default());
    let count: usize = files.len();
    view! {
        <Page mask=true z_index={z_index}>
            <CenterMiddle>
                <Dialog title={Signal::stored(SharedString::from("正在上传"))} closable={false} style={SharedString::from("width: 24em;")} content_style={SharedString::from("max-height: 24em;padding: 0.5em;overflow:auto;")}>
                    <For
                        each=move || { files.clone().into_iter() }
                        key=|(key, _hashing_file, _onsingledone)| { key.clone() }
                        children=move |(_key, hashing_file, onsingledone)| {
                            let done_count = done_count.clone();
                            let onsingledone = onsingledone.clone();
                            let first_error_result = first_error_result.clone();
                            let ondone = UnsyncCallback::new(move |result: Result<ResourceMetadata, SharedString>| {
                                let has_error = first_error_result.read().is_err();
                                if let Err(error) = result.as_ref() {
                                    if !has_error {
                                        first_error_result.set(Err(error.clone()));
                                    }
                                }
                                onsingledone.run(result.clone());
                                if count == 1 + done_count.fetch_add(1, Ordering::Relaxed) {
                                    if has_error {
                                        ondone.run(first_error_result.get());
                                    } else {
                                        ondone.run(result.map(|_| ()));
                                    }
                                }
                            });
                            view! {
                                <UploadingFile file={hashing_file.get()} ondone={ondone}/>
                            }
                        }
                    />
                </Dialog>
            </CenterMiddle>
        </Page>
    }
}

pub async fn upload_files(
    files: Vec<(
        HashingFile,
        UnsyncCallback<Result<ResourceMetadata, SharedString>>,
    )>,
) -> Result<(), SharedString> {
    if files.is_empty() {
        return Ok(());
    }
    let mut files = Some(files);
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let root = document.create_element("div").unwrap();
    let root_clone = root.clone();
    let handle: Rc<Cell<Option<UnmountHandle<AnyViewState>>>> = Rc::new(Cell::new(None));
    let handle_clone = handle.clone();
    let mut promise_fn = move |resolve: Function, reject: Function| {
        body.append_child(&root).unwrap();
        let files = files.take().unwrap_or_default();
        let renderer = move || {
            view! {
                <UploadingFiles
                    files=files
                        .into_iter()
                        .map(|(hashing_file, ondone)| (gen_id().into(), Signal::stored_local(hashing_file), ondone))
                        .collect()
                    z_index=999
                    ondone=UnsyncCallback::new(move |result: Result<(), SharedString>| match result {
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
                    })
                />
            }
            .into_any()
        };
        handle.set(Some(leptos::mount::mount_to(
            root.clone().unchecked_into(),
            renderer,
        )));
    };
    let promise = Promise::new(&mut promise_fn);
    let result = JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|err| -> SharedString {
            log::error!("上传文件失败: {:?}", err);
            return "上传文件失败".into();
        });
    if let Some(handle) = handle_clone.take() {
        // handle.destroy();
        let body = document.body().unwrap();
        body.remove_child(&root_clone).unwrap();
    }
    return result;
}
