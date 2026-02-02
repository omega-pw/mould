use super::HashingFile;
use super::ResourceMetadata;
use crate::utils;
use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn UploadingFile(
    file: HashingFile,
    ondone: UnsyncCallback<Result<ResourceMetadata, SharedString>>,
) -> impl IntoView {
    let progress: RwSignal<f64> = RwSignal::new(0.0);
    {
        let progress = progress.clone();
        let ondone = ondone.clone();
        let hashing_file = file.clone();
        wasm_bindgen_futures::spawn_local(async move {
            progress.set(0.0);
            let on_upload_progress = Box::new(move |loaded: f64, total: f64| {
                progress.set(if 0.0 >= total { 0.0 } else { loaded / total });
            }) as Box<dyn FnMut(f64, f64)>;
            let result = utils::upload_file(
                hashing_file.file.clone(),
                hashing_file.sha512.clone(),
                Some(on_upload_progress),
            )
            .await
            .map(|resp| ResourceMetadata {
                key: resp.key,
                name: hashing_file.file.name(),
                size: hashing_file.file.size(),
                mime_type: hashing_file.file.type_(),
            });
            ondone.run(result);
        });
    }
    let style = move || {
        let progress_percent = (progress.get() * 1000000.0).round() / 10000.0;
        format!("background-color:#CCC;height: 0.25em;border-radius: 2px;background-size: {}% 100%;background-image: linear-gradient(0deg, green 0%, green 100%);background-repeat: no-repeat;background-position: 0 100%;margin-top: 0.25em;", progress_percent)
    };
    view! {
        <div>
            <div>{file.file.name()}</div>
            <div style={style}></div>
        </div>
    }
}
