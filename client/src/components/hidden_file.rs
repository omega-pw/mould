use crate::SharedString;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::File;
use web_sys::HtmlInputElement;

#[component]
pub fn HiddenFile(
    #[prop(into, optional)] root_style: SharedString,
    #[prop(into, optional)] style: SharedString,
    #[prop(into, optional)] accept: SharedString,
    #[prop(into, default = None)] onfiles: Option<UnsyncCallback<Vec<File>>>,
    children: Children,
) -> impl IntoView {
    let file_ref: NodeRef<html::Input> = NodeRef::new();
    let on_file_change = move |evt: Event| {
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
                        if let Some(onfiles) = onfiles.as_ref() {
                            onfiles.run(files);
                        }
                    }
                }
                Err(err) => {
                    log::error!("{:?}", err);
                }
            }
        }
    };
    let on_click = {
        let file_ref = file_ref.clone();
        move |_| {
            if let Some(input_dom) = file_ref.get() {
                input_dom.click();
            }
        }
    };
    view! {
        <div style={root_style}>
            <input node_ref={file_ref} type="file" on:change={on_file_change} accept={accept} style="display:none;"/>
            <div on:click={on_click} style={style}>
                { children() }
            </div>
        </div>
    }
}
