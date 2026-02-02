use super::button::Button;
use super::button_group::ButtonGroup;
use super::input::Input;
use super::modal_dialog::ModalDialog;
use super::on_cleanup_unsync;
use super::uploading_files::upload_files;
use super::validate_wrapper::ValidateData;
use super::validate_wrapper::ValidateWrapper;
use super::HashingFile;
use super::LatestDestroy;
use super::ResourceMetadata;
use crate::utils::choose_file;
use crate::utils::validator;
use crate::utils::validator::Validator;
use crate::utils::validator::Validators;
use crate::SharedString;
use js_sys::Function;
use js_sys::Promise;
use js_sys::JSON;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyViewState;
use std::cell::Cell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::DocumentFragment;
use web_sys::Event;
use web_sys::File;
use web_sys::HtmlElement;
use web_sys::MouseEvent;

#[derive(Clone, PartialEq)]
pub struct LinkInfo {
    pub url: SharedString,
    pub title: Option<SharedString>,
}

#[component]
pub fn LinkPicker(ondone: UnsyncCallback<LinkInfo>, oncancel: UnsyncCallback<()>) -> impl IntoView {
    let url_state: RwSignal<SharedString> = RwSignal::new(Default::default());
    let url = ValidateData::from_state(
        url_state.clone().into(),
        Some(RwSignal::new(Default::default())),
        Some(Validators::new().add(validator::RequiredValidator::new("Please input url"))),
    );
    let title: RwSignal<SharedString> = RwSignal::new(Default::default());
    let on_confirm = {
        let url_state = url_state.clone();
        let title = title.clone();
        UnsyncCallback::new(move |_| {
            let url_state = url_state.read();
            if !url_state.is_empty() {
                let title = title.read();
                let link_info = LinkInfo {
                    url: url_state.clone(),
                    title: if title.is_empty() {
                        None
                    } else {
                        Some(title.clone())
                    },
                };
                ondone.run(link_info);
            }
        })
    };
    view! {
        <ModalDialog title={Signal::stored(SharedString::from("编辑链接"))} closable={false} content_style={SharedString::from("padding-right:2em;padding-top: 1em;padding-bottom: 1em;")}>
            <table style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:4em;vertical-align: top;"><span style="color:red;margin-right: 0.25em;">{"*"}</span>{"Url:"}</td>
                    <td>
                        <ValidateWrapper error={url.error()}>
                            <Input value={url.data().into()} onupdate={url.listener()}/>
                        </ValidateWrapper>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:4em;vertical-align: top;">{"Title:"}</td>
                    <td>
                        <Input value={title}/>
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td style="padding-top: 1em;">
                        <ButtonGroup>
                            <Button disabled={Signal::derive(move || url_state.read().is_empty())} onclick={on_confirm}>{"Confirm"}</Button>
                            <Button onclick={oncancel}>{"Cancel"}</Button>
                        </ButtonGroup>
                    </td>
                </tr>
            </table>
        </ModalDialog>
    }
}

fn get_link() -> Promise {
    let mut promise_fn = move |resolve: Function, _reject: Function| {
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let root = document.create_element("div").unwrap();
        let on_root_click: Function = Closure::wrap(Box::new(|event: Event| {
            event.stop_propagation();
        }) as Box<dyn FnMut(Event)>)
        .into_js_value()
        .dyn_into()
        .unwrap();
        root.add_event_listener_with_callback("mousedown", &on_root_click)
            .unwrap();
        body.append_child(&root).unwrap();
        let handle: Rc<Cell<Option<UnmountHandle<AnyViewState>>>> = Rc::new(Cell::new(None));
        let renderer = {
            let handle = handle.clone();
            let root = root.clone();
            move || {
                view! {
                    <LinkPicker
                        ondone={
                            let root = root.clone();
                            let on_root_click = on_root_click.clone();
                            let handle = handle.clone();
                            let resolve = resolve.clone();
                            UnsyncCallback::new(move |link_info: LinkInfo| {
                                let object = js_sys::Object::new();
                                js_sys::Reflect::set(
                                    &object,
                                    &JsValue::from_str("href"),
                                    &JsValue::from_str(&link_info.url),
                                )
                                .unwrap();
                                if let Some(title) = link_info.title.as_ref() {
                                    js_sys::Reflect::set(
                                        &object,
                                        &JsValue::from_str("title"),
                                        &JsValue::from_str(title),
                                    )
                                    .unwrap();
                                }
                                if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &object) {
                                    log::error!("调用Promise的resolve失败: {:?}", err);
                                }
                                if let Some(handle) = handle.take() {
                                    // handle.destroy();
                                    let document = web_sys::window().unwrap().document().unwrap();
                                    let body = document.body().unwrap();
                                    root.remove_event_listener_with_callback("mousedown", &on_root_click)
                                        .unwrap();
                                    body.remove_child(&root).unwrap();
                                }
                            })
                        }
                        oncancel={
                            let root = root.clone();
                            let handle = handle.clone();
                            UnsyncCallback::new(move |_: ()| {
                                if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &JsValue::UNDEFINED) {
                                    log::error!("调用Promise的resolve失败: {:?}", err);
                                }
                                if let Some(handle) = handle.take() {
                                    // handle.destroy();
                                    let document = web_sys::window().unwrap().document().unwrap();
                                    let body = document.body().unwrap();
                                    root.remove_event_listener_with_callback("mousedown", &on_root_click)
                                        .unwrap();
                                    body.remove_child(&root).unwrap();
                                }
                            })
                        }
                    />
                }
                .into_any()
            }
        };
        handle.set(Some(leptos::mount::mount_to(
            root.unchecked_into(),
            renderer,
        )));
    };
    return Promise::new(&mut promise_fn);
}

const PRESET_COLORS: [[&str; 8]; 2] = [
    [
        "white", "silver", "gray", "black", "maroon", "red", "purple", "fuchsia",
    ],
    [
        "green", "lime", "olive", "yellow", "navy", "blue", "teal", "aqua",
    ],
];

#[component]
pub fn ColorPicker(
    ondone: UnsyncCallback<SharedString>,
    oncancel: UnsyncCallback<()>,
) -> impl IntoView {
    let color_state: RwSignal<SharedString> = RwSignal::new(Default::default());
    let color = ValidateData::from_state(
        color_state.clone().into(),
        Some(RwSignal::new(Default::default())),
        Some(Validators::new().add(validator::RequiredValidator::new("Please input color"))),
    );
    let on_confirm = {
        let color_state = color_state.clone();
        let ondone = ondone.clone();
        UnsyncCallback::new(move |_| {
            let color_state = color_state.read();
            if !color_state.is_empty() {
                ondone.run(color_state.clone());
            }
        })
    };
    view! {
        <ModalDialog title={Signal::stored(SharedString::from("颜色选择"))} closable={false} content_style={SharedString::from("padding:1em;")}>
            <table style="border-spacing: 0.25em;border-collapse: separate;background-color: #EEE;">
                <For
                    each=|| { PRESET_COLORS.iter().enumerate() }
                    key=|(index, _row)| { *index }
                    children={
                        move |(_index, row)| {
                            let ondone = ondone.clone();
                            view! {
                                <tr>
                                    <For
                                        each=move || { row.iter() }
                                        key=|color| { **color }
                                        children=move |color| {
                                            if color.is_empty() {
                                                view! {}.into_any()
                                            } else {
                                                let on_click = move |_evt: MouseEvent| {
                                                    ondone.run(SharedString::Borrowed(color));
                                                };
                                                let style = format!("width: 1.5em;height: 1.5em;cursor:pointer;background-color: {};", color);
                                                view! {
                                                    <td on:click={on_click} title={SharedString::Borrowed(color)} style={style}></td>
                                                }.into_any()
                                            }
                                        }
                                    />
                                </tr>
                            }
                        }
                    }
                />
            </table>
            <div style="margin-top: 1em;">
                <span style="vertical-align: top;">{"Other:"}</span>
                <ValidateWrapper error={color.error()} style={"display: inline-block;margin-left: 0.5em;"}>
                    <Input value={color.data().into()} onupdate={color.listener()} style={SharedString::from("width:8em;")}/>
                </ValidateWrapper>
                <ButtonGroup style={SharedString::from("vertical-align: top;margin-left: 0.5em;")}>
                    <Button disabled={Signal::derive(move || color_state.read().is_empty())} onclick={on_confirm}>{"Confirm"}</Button>
                    <Button onclick={oncancel}>{"Cancel"}</Button>
                </ButtonGroup>
            </div>
        </ModalDialog>
    }
}

fn pick_color() -> Promise {
    let mut promise_fn = move |resolve: Function, _reject: Function| {
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let root = document.create_element("div").unwrap();
        let on_root_click: Function = Closure::wrap(Box::new(|event: Event| {
            event.stop_propagation();
        }) as Box<dyn FnMut(Event)>)
        .into_js_value()
        .dyn_into()
        .unwrap();
        root.add_event_listener_with_callback("mousedown", &on_root_click)
            .unwrap();
        body.append_child(&root).unwrap();
        let handle: Rc<Cell<Option<UnmountHandle<AnyViewState>>>> = Rc::new(Cell::new(None));
        let renderer = {
            let handle = handle.clone();
            let root = root.clone();
            move || {
                view! {
                    <ColorPicker
                        ondone={
                            let root = root.clone();
                            let on_root_click = on_root_click.clone();
                            let handle = handle.clone();
                            let resolve = resolve.clone();
                            UnsyncCallback::new(move |color: SharedString| {
                                if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &JsValue::from_str(&color))
                                {
                                    log::error!("调用Promise的resolve失败: {:?}", err);
                                }
                                if let Some(handle) = handle.take() {
                                    // handle.destroy();
                                    let document = web_sys::window().unwrap().document().unwrap();
                                    let body = document.body().unwrap();
                                    root.remove_event_listener_with_callback("mousedown", &on_root_click)
                                        .unwrap();
                                    body.remove_child(&root).unwrap();
                                }
                            })
                        }
                        oncancel={
                            let root = root.clone();
                            let handle = handle.clone();
                            UnsyncCallback::new(move |_: ()| {
                                if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &JsValue::UNDEFINED) {
                                    log::error!("调用Promise的resolve失败: {:?}", err);
                                }
                                if let Some(handle) = handle.take() {
                                    // handle.destroy();
                                    let document = web_sys::window().unwrap().document().unwrap();
                                    let body = document.body().unwrap();
                                    root.remove_event_listener_with_callback("mousedown", &on_root_click)
                                        .unwrap();
                                    body.remove_child(&root).unwrap();
                                }
                            })
                        }
                    />
                }
                .into_any()
            }
        };
        handle.set(Some(leptos::mount::mount_to(
            root.unchecked_into(),
            renderer,
        )));
    };
    return Promise::new(&mut promise_fn);
}

#[component]
pub fn RichText(
    value: RwSignal<JsValue, LocalStorage>,
    #[prop(optional)] readonly: bool,
    #[prop(into, optional)] placeholder: SharedString,
    #[prop(into, optional)] style: SharedString,
    #[prop(into, default = None)] onchange: Option<UnsyncCallback<JsValue>>,
    #[prop(into, default = None)] onfocus: Option<UnsyncCallback<()>>,
) -> impl IntoView {
    let div_ref: NodeRef<html::Div> = NodeRef::new();
    let latest_destroy = LatestDestroy::new();
    on_cleanup_unsync({
        let latest_destroy = latest_destroy.clone();
        move || {
            latest_destroy.clear();
        }
    });
    {
        let div_ref = div_ref.clone();
        let value = value.clone();
        let editor_opt = RwSignal::new_local(None);
        let inner_value = RwSignal::new_local(get_default_rich_text());
        let on_change = {
            let inner_value = inner_value.clone();
            UnsyncCallback::new(move |new_value: JsValue| {
                inner_value.set(new_value.clone());
                if !readonly {
                    value.set(new_value.clone());
                }
                if let Some(onchange) = &onchange {
                    onchange.run(new_value);
                }
            })
        };
        {
            let editor_opt = editor_opt.clone();
            let placeholder = placeholder.clone();
            let on_change = on_change.clone();
            let on_focus = onfocus.clone();
            Effect::watch(
                move || div_ref.get(),
                move |div_ref, _, _| {
                    if let Some(div) = div_ref {
                        let editor = mount_text_editor(
                            div,
                            value.read().deref(),
                            &placeholder,
                            on_change,
                            on_focus,
                        )
                        .unwrap();
                        editor_opt.set(Some(editor.clone()));
                        latest_destroy.replace(move || {
                            editor_opt.set(None);
                            unmount_text_editor(&editor);
                        });
                    }
                },
                false,
            );
        }
        {
            let value = value.clone();
            let placeholder = placeholder.clone();
            let on_focus = onfocus.clone();
            Effect::watch(
                move || value.get(),
                move |value, _, _| {
                    let inner_value = inner_value.read();
                    if inner_value.deref() != value {
                        if let Some(editor) = editor_opt.read().as_ref() {
                            update_rich_rext(editor, value, &placeholder, on_change, on_focus)
                                .unwrap();
                        }
                    }
                },
                false,
            );
        }
    }
    view! {
        <div node_ref={div_ref} class="rich-text" style={style}></div>
    }
}

fn build_file_image(file: &File) -> js_sys::Object {
    let image = js_sys::Object::new();
    js_sys::Reflect::set(
        &image,
        &JsValue::from_str("imageType"),
        &JsValue::from_str("File"),
    )
    .unwrap();
    js_sys::Reflect::set(&image, &JsValue::from_str("file"), file).unwrap();
    return image;
}

fn package_resource_url(res_key: JsValue) -> String {
    return format!("/{}", res_key.as_string().unwrap());
}

fn build_config(
    placeholder: &str,
    onchange: UnsyncCallback<JsValue>,
    onfocus: Option<UnsyncCallback<()>>,
) -> js_sys::Object {
    let config = js_sys::Object::new();

    let get_link = Closure::wrap(Box::new(get_link) as Box<dyn FnMut() -> Promise>).into_js_value();
    js_sys::Reflect::set(&config, &JsValue::from_str("getLink"), &get_link).unwrap();

    let pick_color =
        Closure::wrap(Box::new(pick_color) as Box<dyn FnMut() -> Promise>).into_js_value();
    js_sys::Reflect::set(&config, &JsValue::from_str("pickColor"), &pick_color).unwrap();

    let get_resource = Closure::wrap(Box::new(move || -> Promise {
        let mut promise_fn = move |resolve: Function, _reject: Function| {
            choose_file(
                move |files: Option<web_sys::FileList>| {
                    let first_file = if let Some(files) = files {
                        files.get(0)
                    } else {
                        None
                    };
                    if let Some(file) = first_file {
                        let image = build_file_image(&file);
                        if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &image) {
                            log::error!("调用Promise的resolve失败: {:?}", err);
                        }
                    } else {
                        if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &JsValue::UNDEFINED) {
                            log::error!("调用Promise的resolve失败: {:?}", err);
                        }
                    }
                },
                Some(String::from("image/*")),
            );
        };
        return Promise::new(&mut promise_fn);
    }) as Box<dyn FnMut() -> Promise>)
    .into_js_value();
    js_sys::Reflect::set(&config, &JsValue::from_str("getResource"), &get_resource).unwrap();

    let package_resource_url =
        Closure::wrap(Box::new(package_resource_url) as Box<dyn Fn(JsValue) -> String>)
            .into_js_value();
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("packageResourceUrl"),
        &package_resource_url,
    )
    .unwrap();

    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("placeholder"),
        &JsValue::from_str(placeholder),
    )
    .unwrap();

    let on_change = Closure::wrap(Box::new(move |value: JsValue| {
        onchange.run(value);
    }) as Box<dyn Fn(JsValue)>)
    .into_js_value();
    js_sys::Reflect::set(&config, &JsValue::from_str("change"), &on_change).unwrap();

    // js_sys::Reflect::set(&config, &JsValue::from_str("blur"), &on_focus).unwrap();

    if let Some(onfocus) = onfocus {
        let on_focus = Closure::wrap(Box::new(move |_value: JsValue| {
            onfocus.run(());
        }) as Box<dyn Fn(JsValue)>)
        .into_js_value();
        js_sys::Reflect::set(&config, &JsValue::from_str("focus"), &on_focus).unwrap();
    }

    return config;
}

fn mount_text_editor(
    root: &HtmlElement,
    value: &JsValue,
    placeholder: &str,
    onchange: UnsyncCallback<JsValue>,
    onfocus: Option<UnsyncCallback<()>>,
) -> Result<JsValue, JsValue> {
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText")).unwrap();
    let mount_rich_text_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("mountRichText"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let arguments = js_sys::Array::new();
    arguments.push(root);
    let config = build_config(placeholder, onchange, onfocus);
    arguments.push(&config);
    arguments.push(value);
    let editor =
        js_sys::Reflect::apply(&mount_rich_text_method, &rich_text_obj, &arguments).unwrap();
    return Ok(editor);
}

fn unmount_text_editor(editor: &JsValue) {
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText")).unwrap();
    let unmount_rich_text_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("unmountRichText"))
            .unwrap()
            .dyn_into()
            .unwrap();
    unmount_rich_text_method
        .call1(&rich_text_obj, editor)
        .unwrap();
}

pub fn get_default_rich_text() -> JsValue {
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText")).unwrap();
    let get_default_rich_text_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("getDefaultRichText"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let value = get_default_rich_text_method.call0(&rich_text_obj).unwrap();
    return value;
}

pub fn get_default_rich_text_string() -> SharedString {
    let value = get_default_rich_text();
    let value = JSON::stringify(&value).unwrap().as_string().unwrap();
    return value.into();
}

pub fn is_empty_rich_rext(value: &JsValue) -> bool {
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText")).unwrap();
    let is_empty_text_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("isEmptyRichText"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let result = is_empty_text_method.call1(&rich_text_obj, value).unwrap();
    result.as_bool().unwrap_or(true)
}

pub fn render_rich_rext(value: &str) -> Result<DocumentFragment, JsValue> {
    let content = JSON::parse(value)?;
    let package_resource_url =
        Closure::wrap(Box::new(package_resource_url) as Box<dyn Fn(JsValue) -> String>)
            .into_js_value();
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText"))?;
    let render_text_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("renderRichText"))?.dyn_into()?;
    let content = render_text_method.call2(&rich_text_obj, &content, &package_resource_url)?;
    return Ok(content.unchecked_into());
}

pub fn update_rich_rext(
    editor: &JsValue,
    value: &JsValue,
    placeholder: &str,
    onchange: UnsyncCallback<JsValue>,
    onfocus: Option<UnsyncCallback<()>>,
) -> Result<(), JsValue> {
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText"))?;
    let update_text_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("updateRichText"))?.dyn_into()?;
    let arguments = js_sys::Array::new();
    arguments.push(editor);
    let config = build_config(placeholder, onchange, onfocus);
    arguments.push(&config);
    arguments.push(value);
    js_sys::Reflect::apply(&update_text_method, &rich_text_obj, &arguments)?;
    return Ok(());
}

fn try_upload(files: JsValue, resolve: Function, reject: Function) -> Result<(), SharedString> {
    let files: js_sys::Array = files.clone().dyn_into().map_err(|err| {
        log::error!("上传文件参数不是数组: {:?}", err);
        SharedString::from("参数错误！")
    })?;
    let files: Result<Vec<File>, JsValue> = files
        .to_vec()
        .into_iter()
        .map(|file| file.dyn_into())
        .collect();
    let files = files.map_err(|err| {
        log::error!("文件列表里存在不是文件的元素: {:?}", err);
        SharedString::from("参数错误！")
    })?;
    let calc_file_sha512_method: js_sys::Function = js_sys::Reflect::get(
        &web_sys::window().unwrap(),
        &JsValue::from_str("calcFileSha512"),
    )
    .unwrap()
    .dyn_into()
    .unwrap();
    let result_map: Arc<RwLock<HashMap<usize, Result<ResourceMetadata, SharedString>>>> =
        Default::default();
    let files: Vec<(
        HashingFile,
        UnsyncCallback<Result<ResourceMetadata, SharedString>>,
    )> = files
        .to_vec()
        .into_iter()
        .enumerate()
        .map(|(index, file)| {
            let result_map = result_map.clone();
            let sha512 = calc_file_sha512_method
                .call1(&wasm_bindgen::JsValue::UNDEFINED, &file)
                .unwrap()
                .dyn_into()
                .unwrap();
            (
                HashingFile {
                    file: file,
                    sha512: sha512,
                },
                UnsyncCallback::new(move |result| {
                    result_map.write().unwrap().insert(index, result);
                }),
            )
        })
        .collect();
    wasm_bindgen_futures::spawn_local(async move {
        match upload_files(files).await {
            Ok(_) => {
                let mut results: Vec<_> = result_map
                    .read()
                    .unwrap()
                    .iter()
                    .map(|(index, result)| (index.clone(), result.clone()))
                    .collect();
                results.sort_by_key(|item| item.0);
                let arguments = js_sys::Array::new();
                for (_, result) in results {
                    arguments.push(&JsValue::from_str(&result.unwrap().key));
                }
                if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &arguments) {
                    log::error!("调用Promise的resolve失败: {:?}", err);
                }
            }
            Err(err) => {
                if let Err(err) = reject.call1(&JsValue::UNDEFINED, &JsValue::from_str(&err)) {
                    log::error!("调用Promise的reject失败: {:?}", err);
                }
            }
        }
    });
    return Ok(());
}

pub async fn upload_resource(value: &JsValue) -> Result<(), SharedString> {
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText")).unwrap();
    let upload_resource_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("uploadResource"))
            .unwrap()
            .dyn_into()
            .unwrap();
    let upload = Closure::wrap(Box::new(move |files: JsValue| -> Promise {
        let mut promise_fn = move |resolve: Function, reject: Function| {
            if let Err(err) = try_upload(files.clone(), resolve, reject.clone()) {
                if let Err(err) =
                    reject.call1(&wasm_bindgen::JsValue::UNDEFINED, &JsValue::from_str(&err))
                {
                    log::error!("调用Promise的reject失败: {:?}", err);
                }
            }
        };
        return Promise::new(&mut promise_fn);
    }) as Box<dyn FnMut(JsValue) -> Promise>)
    .into_js_value();
    let upload_promise: Promise = upload_resource_method
        .call2(&rich_text_obj, &upload, value)
        .unwrap()
        .dyn_into()
        .unwrap();
    JsFuture::from(upload_promise).await.map_err(|err| {
        log::error!("上传图片失败: {:?}", err);
        SharedString::from("上传图片失败！")
    })?;
    return Ok(());
}

pub struct RequiredValidator {
    message: SharedString,
}

impl RequiredValidator {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Validator<str> for RequiredValidator {
    fn validate(&self, data: &str) -> Option<SharedString> {
        match JSON::parse(data) {
            Ok(json) => {
                if is_empty_rich_rext(&json) {
                    Some(self.message.clone())
                } else {
                    None
                }
            }
            Err(_err) => Some(self.message.clone()),
        }
    }
}

impl Validator<String> for RequiredValidator {
    fn validate(&self, data: &String) -> Option<SharedString> {
        match JSON::parse(data) {
            Ok(json) => {
                if is_empty_rich_rext(&json) {
                    Some(self.message.clone())
                } else {
                    None
                }
            }
            Err(_err) => Some(self.message.clone()),
        }
    }
}

impl Validator<SharedString> for RequiredValidator {
    fn validate(&self, data: &SharedString) -> Option<SharedString> {
        match JSON::parse(data) {
            Ok(json) => {
                if is_empty_rich_rext(&json) {
                    Some(self.message.clone())
                } else {
                    None
                }
            }
            Err(_err) => Some(self.message.clone()),
        }
    }
}

impl Validator<JsValue> for RequiredValidator {
    fn validate(&self, data: &JsValue) -> Option<SharedString> {
        if is_empty_rich_rext(data) {
            Some(self.message.clone())
        } else {
            None
        }
    }
}
