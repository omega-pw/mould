use super::button::Button;
use super::button_group::ButtonGroup;
use super::input::BindingInput;
use super::modal_dialog::ModalDialog;
use super::uploading_files::upload_files;
use super::validate_wrapper::ValidateData;
use super::HashingFile;
use crate::utils::choose_file;
use crate::utils::validator;
use crate::utils::validator::Validator;
use crate::utils::validator::Validators;
use js_sys::Function;
use js_sys::Promise;
use js_sys::JSON;
use std::cell::Cell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::DocumentFragment;
use web_sys::File;
use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct LinkInfo {
    pub url: AttrValue,
    pub title: Option<AttrValue>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct LinkPickerProps {
    pub ondone: Callback<LinkInfo>,
    pub oncancel: Callback<()>,
}

#[function_component]
pub fn LinkPicker(props: &LinkPickerProps) -> Html {
    let url_state: UseStateHandle<AttrValue> = use_state(|| Default::default());
    let url = ValidateData::from_state(
        url_state.clone(),
        Some(use_state(|| Default::default())),
        Some(Validators::new().add(validator::RequiredValidator::new("Please input url"))),
    );
    let title: UseStateHandle<AttrValue> = use_state(|| Default::default());
    let on_confirm = {
        let url_state = url_state.clone();
        let title = title.clone();
        let ondone = props.ondone.clone();
        Callback::from(move |_| {
            if !url_state.is_empty() {
                let link_info = LinkInfo {
                    url: url_state.deref().clone(),
                    title: if title.is_empty() {
                        None
                    } else {
                        Some(title.deref().clone())
                    },
                };
                ondone.emit(link_info);
            }
        })
    };
    html! {
        <ModalDialog title={"编辑链接"} closable={false} content_style="padding-right:2em;padding-top: 1em;padding-bottom: 1em;">
            <table style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:4em;vertical-align: top;"><span style="color:red;margin-right: 0.25em;">{"*"}</span>{"Url:"}</td>
                    <td>
                        {
                            url.view(move |url: UseStateHandle<AttrValue>, validator| {
                                html! {
                                    <BindingInput value={url} onupdate={validator}/>
                                }
                            })
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:4em;vertical-align: top;">{"Title:"}</td>
                    <td>
                        <BindingInput value={title}/>
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td style="padding-top: 1em;">
                        <ButtonGroup>
                            <Button disabled={url_state.is_empty()} onclick={on_confirm}>{"Confirm"}</Button>
                            <Button onclick={props.oncancel.clone()}>{"Cancel"}</Button>
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
        let handle: Rc<Cell<Option<AppHandle<LinkPicker>>>> = Rc::new(Cell::new(None));
        let props = LinkPickerProps {
            ondone: {
                let root = root.clone();
                let on_root_click = on_root_click.clone();
                let handle = handle.clone();
                let resolve = resolve.clone();
                Callback::from(move |link_info: LinkInfo| {
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
                        handle.destroy();
                        let document = web_sys::window().unwrap().document().unwrap();
                        let body = document.body().unwrap();
                        root.remove_event_listener_with_callback("mousedown", &on_root_click)
                            .unwrap();
                        body.remove_child(&root).unwrap();
                    }
                })
            },
            oncancel: {
                let root = root.clone();
                let handle = handle.clone();
                Callback::from(move |_: ()| {
                    if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &JsValue::UNDEFINED) {
                        log::error!("调用Promise的resolve失败: {:?}", err);
                    }
                    if let Some(handle) = handle.take() {
                        handle.destroy();
                        let document = web_sys::window().unwrap().document().unwrap();
                        let body = document.body().unwrap();
                        root.remove_event_listener_with_callback("mousedown", &on_root_click)
                            .unwrap();
                        body.remove_child(&root).unwrap();
                    }
                })
            },
        };
        handle.set(Some(
            yew::Renderer::<LinkPicker>::with_root_and_props(root, props).render(),
        ));
    };
    return Promise::new(&mut promise_fn);
}

#[derive(Clone, PartialEq, Properties)]
pub struct ColorPickerProps {
    pub ondone: Callback<AttrValue>,
    pub oncancel: Callback<()>,
}

const PRESET_COLORS: [[&str; 8]; 2] = [
    [
        "white", "silver", "gray", "black", "maroon", "red", "purple", "fuchsia",
    ],
    [
        "green", "lime", "olive", "yellow", "navy", "blue", "teal", "aqua",
    ],
];

#[function_component]
pub fn ColorPicker(props: &ColorPickerProps) -> Html {
    let color_state: UseStateHandle<AttrValue> = use_state(|| Default::default());
    let color = ValidateData::from_state(
        color_state.clone(),
        Some(use_state(|| Default::default())),
        Some(Validators::new().add(validator::RequiredValidator::new("Please input color"))),
    );
    let on_confirm = {
        let color_state = color_state.clone();
        let ondone = props.ondone.clone();
        Callback::from(move |_| {
            if !color_state.is_empty() {
                ondone.emit(color_state.deref().clone());
            }
        })
    };
    html! {
        <ModalDialog title={"颜色选择"} closable={false} content_style="padding:1em;">
            <table style="border-spacing: 0.25em;border-collapse: separate;background-color: #EEE;">
                {
                    for PRESET_COLORS.iter().map(|row| {
                        html! {
                            <tr>
                                {
                                    for row.iter().map(|color| {
                                        if color.is_empty() {
                                            html! {}
                                        } else {
                                            let ondone = props.ondone.clone();
                                            let on_click = Callback::from(move |_evt: MouseEvent| {
                                                ondone.emit(AttrValue::Static(color));
                                            });
                                            let style = format!("width: 1.5em;height: 1.5em;cursor:pointer;background-color: {};", color);
                                            html! {
                                                <td onclick={on_click} title={AttrValue::Static(color)} style={style}></td>
                                            }
                                        }
                                    })
                                }
                            </tr>
                        }
                    })
                }
            </table>
            <div style="margin-top: 1em;">
                <span style="vertical-align: top;">{"Other:"}</span>
                {
                    color.view_with_style(move |color: UseStateHandle<AttrValue>, validator| {
                        html! {
                            <BindingInput value={color} onupdate={validator} style="width:8em;"/>
                        }
                    }, Some("display: inline-block;margin-left: 0.5em;".into()))
                }
                <ButtonGroup style="vertical-align: top;margin-left: 0.5em;">
                    <Button disabled={color_state.is_empty()} onclick={on_confirm}>{"Confirm"}</Button>
                    <Button onclick={props.oncancel.clone()}>{"Cancel"}</Button>
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
        let handle: Rc<Cell<Option<AppHandle<ColorPicker>>>> = Rc::new(Cell::new(None));
        let props = ColorPickerProps {
            ondone: {
                let root = root.clone();
                let on_root_click = on_root_click.clone();
                let handle = handle.clone();
                let resolve = resolve.clone();
                Callback::from(move |color: AttrValue| {
                    if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &JsValue::from_str(&color))
                    {
                        log::error!("调用Promise的resolve失败: {:?}", err);
                    }
                    if let Some(handle) = handle.take() {
                        handle.destroy();
                        let document = web_sys::window().unwrap().document().unwrap();
                        let body = document.body().unwrap();
                        root.remove_event_listener_with_callback("mousedown", &on_root_click)
                            .unwrap();
                        body.remove_child(&root).unwrap();
                    }
                })
            },
            oncancel: {
                let root = root.clone();
                let handle = handle.clone();
                Callback::from(move |_: ()| {
                    if let Err(err) = resolve.call1(&JsValue::UNDEFINED, &JsValue::UNDEFINED) {
                        log::error!("调用Promise的resolve失败: {:?}", err);
                    }
                    if let Some(handle) = handle.take() {
                        handle.destroy();
                        let document = web_sys::window().unwrap().document().unwrap();
                        let body = document.body().unwrap();
                        root.remove_event_listener_with_callback("mousedown", &on_root_click)
                            .unwrap();
                        body.remove_child(&root).unwrap();
                    }
                })
            },
        };
        handle.set(Some(
            yew::Renderer::<ColorPicker>::with_root_and_props(root, props).render(),
        ));
    };
    return Promise::new(&mut promise_fn);
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: JsValue,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    pub onchange: Callback<JsValue>,
    #[prop_or_default]
    pub onfocus: Callback<()>,
}

#[function_component]
pub fn RichText(props: &Props) -> Html {
    let div_ref = use_node_ref();
    {
        let div_ref = div_ref.clone();
        let value = props.value.clone();
        let editor_opt = use_state(|| None);
        let inner_value = use_state(get_default_rich_text);
        let onchange = props.onchange.clone();
        let on_change = {
            let inner_value = inner_value.clone();
            Callback::from(move |value: JsValue| {
                inner_value.set(value.clone());
                onchange.emit(value);
            })
        };
        {
            let editor_opt = editor_opt.clone();
            let placeholder = props.placeholder.clone();
            let on_change = on_change.clone();
            let on_focus = props.onfocus.clone();
            use_effect_with(div_ref, move |div_ref| {
                let div = div_ref
                    .cast::<HtmlElement>()
                    .expect("div_ref not attached to div element");
                let editor =
                    mount_text_editor(&div, &value, &placeholder, on_change, on_focus).unwrap();
                editor_opt.set(Some(editor.clone()));
                move || {
                    editor_opt.set(None);
                    unmount_text_editor(&editor);
                }
            });
        }
        {
            let value = props.value.clone();
            let placeholder = props.placeholder.clone();
            let on_focus = props.onfocus.clone();
            use_effect_with(value, move |value| {
                if inner_value.deref() != value {
                    if let Some(editor) = editor_opt.as_ref() {
                        update_rich_rext(editor, value, &placeholder, on_change, on_focus).unwrap();
                    }
                }
            });
        }
    }
    html! {
        <div ref={div_ref} style={props.style.clone()}></div>
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
    onchange: Callback<JsValue>,
    onfocus: Callback<()>,
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
        onchange.emit(value);
    }) as Box<dyn Fn(JsValue)>)
    .into_js_value();
    js_sys::Reflect::set(&config, &JsValue::from_str("change"), &on_change).unwrap();

    // js_sys::Reflect::set(&config, &JsValue::from_str("blur"), &on_focus).unwrap();

    let on_focus = Closure::wrap(Box::new(move |_value: JsValue| {
        onfocus.emit(());
    }) as Box<dyn Fn(JsValue)>)
    .into_js_value();
    js_sys::Reflect::set(&config, &JsValue::from_str("focus"), &on_focus).unwrap();

    return config;
}

fn mount_text_editor(
    root: &HtmlElement,
    value: &JsValue,
    placeholder: &str,
    onchange: Callback<JsValue>,
    onfocus: Callback<()>,
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

pub fn get_default_rich_text_string() -> AttrValue {
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

pub fn render_rich_rext(content: &JsValue) -> Result<DocumentFragment, JsValue> {
    let package_resource_url =
        Closure::wrap(Box::new(package_resource_url) as Box<dyn Fn(JsValue) -> String>)
            .into_js_value();
    let rich_text_obj =
        js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("RichText"))?;
    let render_text_method: js_sys::Function =
        js_sys::Reflect::get(&rich_text_obj, &JsValue::from_str("renderRichText"))?.dyn_into()?;
    let content = render_text_method.call2(&rich_text_obj, content, &package_resource_url)?;
    return Ok(content.unchecked_into());
}

pub fn update_rich_rext(
    editor: &JsValue,
    value: &JsValue,
    placeholder: &str,
    onchange: Callback<JsValue>,
    onfocus: Callback<()>,
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

fn try_upload(files: JsValue, resolve: Function, reject: Function) -> Result<(), AttrValue> {
    let files: js_sys::Array = files.clone().dyn_into().map_err(|err| {
        log::error!("上传文件参数不是数组: {:?}", err);
        AttrValue::from("参数错误！")
    })?;
    let files: Result<Vec<File>, JsValue> = files
        .to_vec()
        .into_iter()
        .map(|file| file.dyn_into())
        .collect();
    let files = files.map_err(|err| {
        log::error!("文件列表里存在不是文件的元素: {:?}", err);
        AttrValue::from("参数错误！")
    })?;
    let calc_file_sha512_method: js_sys::Function = js_sys::Reflect::get(
        &web_sys::window().unwrap(),
        &JsValue::from_str("calcFileSha512"),
    )
    .unwrap()
    .dyn_into()
    .unwrap();
    let result_map: Arc<RwLock<HashMap<usize, Result<String, AttrValue>>>> = Default::default();
    let files: Vec<(HashingFile, Callback<Result<String, AttrValue>>)> = files
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
                Callback::from(move |result| {
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
                    arguments.push(&JsValue::from_str(&result.unwrap()));
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

pub async fn upload_resource(value: &JsValue) -> Result<(), AttrValue> {
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
        AttrValue::from("上传图片失败！")
    })?;
    return Ok(());
}

pub struct RequiredValidator {
    message: AttrValue,
}

impl RequiredValidator {
    pub fn new(message: impl Into<AttrValue>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Validator<str> for RequiredValidator {
    fn validate(&self, data: &str) -> Option<AttrValue> {
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
    fn validate(&self, data: &String) -> Option<AttrValue> {
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

impl Validator<AttrValue> for RequiredValidator {
    fn validate(&self, data: &AttrValue) -> Option<AttrValue> {
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
    fn validate(&self, data: &JsValue) -> Option<AttrValue> {
        if is_empty_rich_rext(data) {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct BindingProps {
    pub value: UseStateHandle<JsValue>,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    #[prop_or_default]
    pub onchange: Callback<JsValue>,
    #[prop_or_default]
    pub onfocus: Callback<()>,
}

#[function_component]
pub fn BindingRichText(props: &BindingProps) -> Html {
    let value_clone = props.value.clone();
    let onchange = props.onchange.clone();
    let on_change = Callback::from(move |value: JsValue| {
        value_clone.set(value.clone().into());
        onchange.emit(value);
    });
    return html! {
        <RichText
            value={props.value.deref().clone()}
            placeholder={props.placeholder.clone()}
            style={props.style.clone()}
            onchange={on_change}
            onfocus={props.onfocus.clone()}
        />
    };
}
