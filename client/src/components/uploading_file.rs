use super::HashingFile;
use crate::utils;
use crate::LightString;
use std::ops::Deref;
use yew::prelude::*;
use yew::{html, Html};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub file: HashingFile,
    #[prop_or_default]
    pub ondone: Callback<Result<String, LightString>>,
}

#[function_component]
pub fn UploadingFile(props: &Props) -> Html {
    let progress: UseStateHandle<f64> = use_state(Default::default);
    {
        let progress = progress.clone();
        let ondone = props.ondone.clone();
        use_effect_with(props.file.clone(), move |file| {
            let progress = progress.clone();
            let hashing_file = file.clone();
            let ondone = ondone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                progress.set(0.0);
                let on_upload_progress = Box::new(move |loaded: f64, total: f64| {
                    progress.set(if 0.0 >= total { 0.0 } else { loaded / total });
                }) as Box<dyn FnMut(f64, f64)>;
                let result = utils::upload_file(
                    hashing_file.file,
                    hashing_file.sha512,
                    Some(on_upload_progress),
                )
                .await
                .map(|resp| resp.key);
                ondone.emit(result);
            });
        });
    }
    let progress_percent = (progress.deref() * 1000000.0).round() / 10000.0;
    let style = format!("background-color:#CCC;height: 0.25em;border-radius: 2px;background-size: {}% 100%;background-image: linear-gradient(0deg, green 0%, green 100%);background-repeat: no-repeat;background-position: 0 100%;margin-top: 0.25em;", progress_percent);
    html! {
        <div>
            <div>{props.file.file.name()}</div>
            <div style={style}></div>
        </div>
    }
}
