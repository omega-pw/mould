use super::center_middle::CenterMiddle;
use super::dialog::Dialog;
use super::page::Page;
use crate::SharedString;
use leptos::prelude::*;

#[component]
pub fn ModalDialog(
    #[prop(into)] title: Signal<SharedString>,
    closable: bool,
    #[prop(default = 1)] z_index: u64,
    #[prop(into, optional)] center_style: SharedString,
    #[prop(into, optional)] dialog_style: SharedString,
    #[prop(into, optional)] content_style: SharedString,
    #[prop(into, default = None)] onclose: Option<UnsyncCallback<()>>,
    children: Children,
) -> impl IntoView {
    let mut actual_content_style = String::from("background-color:#FFF;");
    if !content_style.is_empty() {
        actual_content_style.push_str(&content_style);
    }
    view! {
        <Page mask=true z_index={z_index}>
            <CenterMiddle content_style={center_style.clone()}>
                <Dialog title={title.clone()} closable={closable} onclose={onclose} style={dialog_style.clone()} content_style={actual_content_style}>
                    { children() }
                </Dialog>
            </CenterMiddle>
        </Page>
    }
}
