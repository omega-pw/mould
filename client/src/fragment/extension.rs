use crate::components::button::Button;
use crate::components::checkbox::Checkbox;
use crate::components::checkbox_group::CheckboxGroup;
use crate::components::file_upload::FileUpload;
use crate::components::files_upload::FilesUpload;
use crate::components::input::Input;
use crate::components::monaco_editor::MonacoEditor;
use crate::components::radio_group::RadioGroup;
use crate::components::required::Required;
use crate::components::validate_wrapper::ValidateWrapper;
// use crate::components::rich_text::render_rich_rext;
// use crate::components::rich_text::RichText;
use crate::components::textarea::Textarea;
use crate::components::validate_wrapper::ValidateData;
use crate::components::Resource;
use crate::components::ResourceMetadata;
use crate::sdk;
use crate::utils::gen_id;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::SharedString;
// use js_sys::JSON;
use sdk::extension::Attribute;
use sdk::extension::AttributeType;
use sdk::extension::Extension;
use sdk::extension::Operation;
use serde_json::Value;
// use wasm_bindgen::prelude::*;
use crate::Key;
use leptos::prelude::*;
use web_sys::DocumentFragment;

#[derive(Clone)]
pub enum AttributeValue {
    String(ValidateData<SharedString>),
    StringList(ValidateData<Vec<(Key, RwSignal<SharedString>)>>),
    LongString(ValidateData<SharedString>),
    // RichText(ValidateData<JsValue>),
    Code(ValidateData<SharedString>),
    Password(ValidateData<SharedString>),
    Enum(ValidateData<Option<SharedString>>),
    EnumList(ValidateData<Vec<SharedString>>),
    Bool(RwSignal<bool>),
    File(ValidateData<Option<Resource>, LocalStorage>),
    FileList(ValidateData<Vec<(Key, RwSignal<Resource, LocalStorage>, ())>, LocalStorage>),
}

impl AttributeValue {
    pub fn validate(&self, update_view: bool) -> Result<(), SharedString> {
        match self {
            AttributeValue::String(value) => value.validate(update_view),
            AttributeValue::StringList(value) => value.validate(update_view),
            AttributeValue::LongString(value) => value.validate(update_view),
            // AttributeValue::RichText(value) => value.validate(update_view),
            AttributeValue::Code(value) => value.validate(update_view),
            AttributeValue::Password(value) => value.validate(update_view),
            AttributeValue::Enum(value) => value.validate(update_view),
            AttributeValue::EnumList(value) => value.validate(update_view),
            AttributeValue::Bool(_value) => Ok(()),
            AttributeValue::File(value) => value.validate(update_view),
            AttributeValue::FileList(value) => value.validate(update_view),
        }
    }
}

#[component]
pub fn ConfigView(
    #[prop(into)] attributes: Signal<Vec<(Key, Attribute, AttributeValue)>>,
) -> impl IntoView {
    view! {
        <For
            each={
                let attributes = attributes.clone();
                move || { attributes.get().into_iter() }
            }
            key=|(key, _attribute, _value)| { key.clone() }
            children=move |(_key, attribute, value)| {
                match &attribute.r#type {
                    AttributeType::String => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::String(value) => {
                                                view! {
                                                    <ValidateWrapper error={value.error()}>
                                                        <Input value={value.data()} onupdate={value.listener()}/>
                                                    </ValidateWrapper>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    },
                    AttributeType::StringList => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::StringList(value_list) => {
                                                let validator = value_list.listener();
                                                let value_list_clone = value_list.data();
                                                view! {
                                                    <div>
                                                        <For
                                                            each={
                                                                let value_list = value_list.data();
                                                                move || { value_list.get().into_iter().enumerate() }
                                                            }
                                                            key=|(_index, (key, _value))| { key.clone() }
                                                            children=move |(index, (_key, value))| {
                                                                let on_update = {
                                                                    let value_list = value_list_clone.clone();
                                                                    let validator = validator.clone();
                                                                    UnsyncCallback::new(move |new_value: SharedString| {
                                                                        value.set(new_value);
                                                                        validator.run(value_list.get());
                                                                    })
                                                                };
                                                                let on_remove = {
                                                                    let value_list = value_list_clone.clone();
                                                                    let validator = validator.clone();
                                                                    UnsyncCallback::new(move |_| {
                                                                        value_list.write().remove(index);
                                                                        validator.run(value_list.get());
                                                                    })
                                                                };
                                                                view! {
                                                                    <div>
                                                                        <Input value={value.clone()} onupdate={on_update}/>
                                                                        <Button onclick={on_remove}>{"Remove"}</Button>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                        <Button onclick={UnsyncCallback::new(move |_| {
                                                            value_list_clone.write().push((gen_id().into(), RwSignal::new(SharedString::from(""))));
                                                            validator.run(value_list_clone.get());
                                                        })}>{"Add"}</Button>
                                                    </div>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    },
                    AttributeType::LongString => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::LongString(value) => {
                                                view! {
                                                    <ValidateWrapper error={value.error()}>
                                                        <Textarea value={value.data()} onupdate={value.listener()} style="width:100%;"/>
                                                    </ValidateWrapper>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    },
                    // AttributeType::RichText => {
                    //     let title = if let Some(description) = attribute.description.as_ref() {
                    //         format!("{}({})", attribute.name, description)
                    //     } else {
                    //         format!("{}", attribute.name)
                    //     };
                    //     view! {
                    //         <div>
                    //             <div>
                    //                 <If condition={attribute.required}><Required/></If>
                    //                 { title }
                    //             </div>
                    //             <div>
                    //                 {
                    //                     match value {
                    //                         AttributeValue::RichText(value) => {
                    //                             value.view(move |value, validator: UnsyncCallback<JsValue>| {
                    //                                 view! {
                    //                                     <RichText value={value} onchange={validator} style="border: 1px solid rgba(0, 0, 0, 0.2);padding: 0.25em 0;min-height: 8em;"/>
                    //                                 }
                    //                             })
                    //                         },
                    //                         _ => view!{}
                    //                     }
                    //                 }
                    //             </div>
                    //         </div>
                    //     }
                    // },
                    AttributeType::Code { language } => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::Code(value) => {
                                                let language = SharedString::from(language.clone());
                                                view! {
                                                    <ValidateWrapper error={value.error()}>
                                                        <MonacoEditor value={value.data()} language={language} width=SharedString::from("100%") height=SharedString::from("16em") onchange={value.listener()}/>
                                                    </ValidateWrapper>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    },
                    AttributeType::Password => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::Password(value) => {
                                                view! {
                                                    <ValidateWrapper error={value.error()}>
                                                        <Input r#type="password" disable_trim={true} value={value.data()} onupdate={value.listener()}/>
                                                    </ValidateWrapper>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    },
                    AttributeType::Enum { options } => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::Enum(value) => {
                                                let options = options.clone();
                                                let options: Vec<_> = options.iter().map(|option| {
                                                    (option.value.clone().into(), option.label.clone())
                                                }).collect();
                                                let onchange = {
                                                    let value = value.clone();
                                                    UnsyncCallback::new(move |_| {
                                                        value.validate(true);
                                                    })
                                                };
                                                view! {
                                                    <ValidateWrapper error={value.error()}>
                                                        <RadioGroup value={value.data()} options={options} onchange={onchange}/>
                                                    </ValidateWrapper>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    },
                    AttributeType::EnumList { options } => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::EnumList(values) => {
                                                let options: Vec<_> = options.iter().map(|option| {
                                                    (option.value.clone().into(), option.label.clone())
                                                }).collect();
                                                view! {
                                                    <ValidateWrapper error={values.error()}>
                                                        <CheckboxGroup value={values.data()} options={options} onchange={values.listener()}/>
                                                    </ValidateWrapper>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    },
                    AttributeType::Bool => {
                        let name = attribute.name.clone();
                        let description = if let Some(description) = attribute.description.as_ref() {
                            format!("({})", description)
                        } else {
                            String::from("")
                        };
                        view! {
                            <div>
                                {
                                    match value {
                                        AttributeValue::Bool(value) => {
                                            view! {
                                                <Checkbox value={value} label={name} />
                                                { description }
                                            }.into_any()
                                        },
                                        _ => view!{}.into_any()
                                    }
                                }
                            </div>
                        }.into_any()
                    }
                    AttributeType::File => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::File(file) => {
                                                view! {
                                                    <FileUpload file={file.data()} onchange={file.listener()}/>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    }
                    AttributeType::FileList => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>
                                    <Show when={
                                        let required = attribute.required;
                                        move || required
                                    }>
                                        <Required/>
                                    </Show>
                                    { title }
                                </div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::FileList(files) => {
                                                view! {
                                                    <FilesUpload files={files.data()} onchange={files.listener()}/>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }.into_any()
                    }
                }
            }
        />
    }
}

#[component]
pub fn ConfigDetailView(
    #[prop(into)] attributes: Signal<Vec<(Key, Attribute, AttributeValue)>>,
) -> impl IntoView {
    view! {
        <For
            each={
                let attributes = attributes.clone();
                move || { attributes.get().into_iter() }
            }
            key=|(key, _attribute, _value)| { key.clone() }
            children=move |(_key, attribute, value)| {
                match &attribute.r#type {
                    AttributeType::String => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::String(value) => {
                                                value.data().into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    AttributeType::StringList => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::StringList(value) => {
                                                view! {
                                                    <For
                                                        each={
                                                            let value_list = value.data();
                                                            move || { value_list.get() }
                                                        }
                                                        key=|(key, _value)| { key.clone() }
                                                        children=move |(_key, value)| {
                                                            view! {
                                                                <div>{value}</div>
                                                            }
                                                        }
                                                    />
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    AttributeType::LongString => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::LongString(value) => {
                                                value.data().into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    // AttributeType::RichText => {
                    //     let title = if let Some(description) = attribute.description.as_ref() {
                    //         format!("{}({})", attribute.name, description)
                    //     } else {
                    //         format!("{}", attribute.name)
                    //     };
                    //     view! {
                    //         <div>
                    //             <div>{ title }</div>
                    //             <div>
                    //                 {
                    //                     match value {
                    //                         AttributeValue::RichText(value) => {
                    //                             value.view(move |value: RwSignal<JsValue>, _validator: UnsyncCallback<JsValue>| {
                    //                                 let content = render_rich_rext(&value).unwrap();
                    //                                 wrap_content(content)
                    //                             })
                    //                         },
                    //                         _ => view!{}
                    //                     }
                    //                 }
                    //             </div>
                    //         </div>
                    //     }
                    // },
                    AttributeType::Code { language } => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::Code(value) => {
                                                let language = SharedString::from(language.clone());
                                                view! {
                                                    <MonacoEditor value={value.data()} language={language} readonly={true} width={SharedString::from("100%")} height={SharedString::from("16em")}/>
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    AttributeType::Password => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::Password(value) => {
                                                let value = value.data();
                                                (move || {
                                                    let len = value.read().chars().count();
                                                    "*".repeat(len).into_any()
                                                }).into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    AttributeType::Enum { options } => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::Enum(value) => {
                                                let options = options.clone();
                                                let value = value.data();
                                                (move || {
                                                    let value = value.read();
                                                    let label = options.iter().find(|option| {
                                                        value.as_deref() == Some(option.value.as_str())
                                                    }).map(|option| option.label.clone()).unwrap_or_default();
                                                    label
                                                }).into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    AttributeType::EnumList { options } => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::EnumList(values) => {
                                                let options = options.clone();
                                                let values = values.data();
                                                (move || {
                                                    let values = values.read();
                                                    let labels: Vec<&str> = options.iter().filter(|option| {
                                                        values.iter().any(|value| value == &option.value)
                                                    }).map(|option| option.label.as_ref()).collect();
                                                    labels.join(",")
                                                }).into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    AttributeType::Bool => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::Bool(value) => {
                                                (move || {
                                                    if value.get() {
                                                        "是"
                                                    } else {
                                                        "否"
                                                    }
                                                }).into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    }
                    AttributeType::File => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::File(file) => {
                                                let file = file.data();
                                                (move || {
                                                    if let Some(file) = file.get() {
                                                        match file {
                                                            Resource::Remote(metadata) => {
                                                                let url = format!("/{}", metadata.key);
                                                                view! {
                                                                    <a href={url} target="_blank">{metadata.name}</a>
                                                                }.into_any()
                                                            }
                                                            Resource::Local(hashing_file) => {
                                                                hashing_file.file.name().into_any()
                                                            }
                                                        }
                                                    } else {
                                                        view! {}.into_any()
                                                    }
                                                }).into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                    AttributeType::FileList => {
                        let title = if let Some(description) = attribute.description.as_ref() {
                            format!("{}({})", attribute.name, description)
                        } else {
                            format!("{}", attribute.name)
                        };
                        view! {
                            <div>
                                <div>{ title }</div>
                                <div>
                                    {
                                        match value {
                                            AttributeValue::FileList(files) => {
                                                let files = files.data();
                                                view! {
                                                    <For
                                                        each={
                                                            let files = files.clone();
                                                            move || { files.get() }
                                                        }
                                                        key=|(key, _file, _)| { key.clone() }
                                                        children=move |(_key, file, _)| {
                                                            (move || {
                                                                match file.get() {
                                                                    Resource::Remote(metadata) => {
                                                                        let url = format!("/{}", metadata.key);
                                                                        view! {
                                                                            <a href={url} target="_blank">{metadata.name}</a>
                                                                        }.into_any()
                                                                    }
                                                                    Resource::Local(hashing_file) => {
                                                                        hashing_file.file.name().into_any()
                                                                    }
                                                                }
                                                            }).into_any()
                                                        }
                                                    />
                                                }.into_any()
                                            },
                                            _ => view!{}.into_any()
                                        }
                                    }
                                </div>
                            </div>
                        }
                    },
                }
            }
        />
    }
}

pub fn parse_config(
    attributes: Vec<Attribute>,
    config: &str,
) -> Vec<(Key, Attribute, AttributeValue)> {
    let config = serde_json::from_str::<Value>(config).unwrap_or_else(|err| {
        log::error!("配置格式不正确：{}", err);
        Value::Object(serde_json::Map::new())
    });
    let mut config = match config {
        Value::Object(config) => config,
        _ => serde_json::Map::new(),
    };
    let mut list = Vec::with_capacity(attributes.len());
    for attribute in attributes {
        let value = config.remove(&attribute.id);
        if let Some(value) = value {
            let value = get_value(&attribute, value);
            list.push((gen_id().into(), attribute, value));
        } else {
            let default_value = get_default_value(&attribute);
            list.push((gen_id().into(), attribute, default_value));
        }
    }
    return list;
}

pub fn get_default_config(attributes: Vec<Attribute>) -> Vec<(Key, Attribute, AttributeValue)> {
    return attributes
        .into_iter()
        .map(|attribute| {
            let default_value = get_default_value(&attribute);
            (gen_id().into(), attribute, default_value)
        })
        .collect();
}

pub fn serialize_config(attributes: &[(Key, Attribute, AttributeValue)]) -> String {
    let mut config = serde_json::Map::new();
    for (_key, attribute, value) in attributes {
        let key = attribute.id.clone();
        let value = match value {
            AttributeValue::String(value) => Value::String(value.get().to_string()),
            AttributeValue::StringList(value) => Value::Array(
                value
                    .get()
                    .into_iter()
                    .map(|(_, value)| Value::String(value.read().to_string()))
                    .collect(),
            ),
            AttributeValue::LongString(value) => Value::String(value.get().to_string()),
            // AttributeValue::RichText(value) => {
            //     Value::String(JSON::stringify(&value.get()).unwrap().as_string().unwrap())
            // }
            AttributeValue::Code(value) => Value::String(value.get().to_string()),
            AttributeValue::Password(value) => Value::String(value.get().to_string()),
            AttributeValue::Enum(value) => {
                if let Some(value) = value.get() {
                    Value::String(value.to_string())
                } else {
                    Value::Null
                }
            }
            AttributeValue::EnumList(values) => Value::Array(
                values
                    .get()
                    .into_iter()
                    .map(|value| Value::String(value.to_string()))
                    .collect(),
            ),
            AttributeValue::Bool(value) => Value::Bool(value.get()),
            AttributeValue::File(value) => {
                let file = value.get();
                if let Some(file) = file {
                    match file {
                        Resource::Remote(metadata) => {
                            let mut map = serde_json::Map::new();
                            map.insert(String::from("key"), Value::String(metadata.key));
                            map.insert(String::from("name"), Value::String(metadata.name));
                            map.insert(String::from("size"), Value::from(metadata.size));
                            map.insert(
                                String::from("mime_type"),
                                Value::String(metadata.mime_type),
                            );
                            Value::Object(map)
                        }
                        Resource::Local(_) => {
                            unreachable!();
                        }
                    }
                } else {
                    Value::Null
                }
            }
            AttributeValue::FileList(value) => Value::Array(
                value
                    .get()
                    .into_iter()
                    .map(|(_, file, _)| match file.get() {
                        Resource::Remote(metadata) => {
                            let mut map = serde_json::Map::new();
                            map.insert(String::from("key"), Value::String(metadata.key));
                            map.insert(String::from("name"), Value::String(metadata.name));
                            map.insert(String::from("size"), Value::from(metadata.size));
                            map.insert(
                                String::from("mime_type"),
                                Value::String(metadata.mime_type),
                            );
                            Value::Object(map)
                        }
                        Resource::Local(_) => {
                            unreachable!();
                        }
                    })
                    .collect(),
            ),
        };
        config.insert(key, value);
    }
    return serde_json::to_string(&Value::Object(config))
        .map_err(|err| {
            log::error!("序列化配置失败：{}", err);
            err
        })
        .unwrap();
}

fn get_string_validators(attribute: &Attribute) -> Option<Validators<SharedString>> {
    return if attribute.required && AttributeType::Bool != attribute.r#type {
        Some(Validators::new().add(RequiredValidator::new(format!("请输入{}", attribute.name))))
    } else {
        None
    };
}

fn get_string_list_validators(
    attribute: &Attribute,
) -> Option<Validators<Vec<(Key, RwSignal<SharedString>)>>> {
    return if attribute.required && AttributeType::Bool != attribute.r#type {
        Some(Validators::new().add(RequiredValidator::new(format!("请输入{}", attribute.name))))
    } else {
        None
    };
}

fn get_file_list_validators(
    attribute: &Attribute,
) -> Option<Validators<Vec<(Key, RwSignal<Resource, LocalStorage>, ())>>> {
    return if attribute.required && AttributeType::Bool != attribute.r#type {
        Some(Validators::new().add(RequiredValidator::new(format!(
            "请上传文件{}",
            attribute.name
        ))))
    } else {
        None
    };
}

fn get_file_validators(attribute: &Attribute) -> Option<Validators<Option<Resource>>> {
    return if attribute.required && AttributeType::Bool != attribute.r#type {
        Some(Validators::new().add(RequiredValidator::new(format!(
            "请上传文件{}",
            attribute.name
        ))))
    } else {
        None
    };
}

fn get_default_value(attribute: &Attribute) -> AttributeValue {
    let validators = get_string_validators(attribute);
    match attribute.r#type {
        AttributeType::String => {
            AttributeValue::String(ValidateData::new(Default::default(), validators))
        }
        AttributeType::StringList => AttributeValue::StringList(ValidateData::new(
            Default::default(),
            get_string_list_validators(attribute),
        )),
        AttributeType::LongString => {
            AttributeValue::LongString(ValidateData::new(Default::default(), validators))
        }
        // AttributeType::RichText => AttributeValue::RichText(ValidateData::new(
        //     Default::default(),
        //     if attribute.required && AttributeType::Bool != attribute.r#type {
        //         Some(
        //             Validators::new().add(crate::components::rich_text::RequiredValidator::new(
        //                 format!("请输入{}", attribute.name),
        //             )),
        //         )
        //     } else {
        //         None
        //     },
        // )),
        AttributeType::Code { .. } => {
            AttributeValue::Code(ValidateData::new(Default::default(), validators))
        }
        AttributeType::Password => {
            AttributeValue::Password(ValidateData::new(Default::default(), validators))
        }
        AttributeType::Enum { .. } => AttributeValue::Enum(ValidateData::new(
            Default::default(),
            if attribute.required {
                Some(
                    Validators::new()
                        .add(RequiredValidator::new(format!("请选择{}", attribute.name))),
                )
            } else {
                None
            },
        )),
        AttributeType::EnumList { .. } => AttributeValue::EnumList(ValidateData::new(
            Default::default(),
            if attribute.required {
                Some(
                    Validators::new()
                        .add(RequiredValidator::new(format!("请选择{}", attribute.name))),
                )
            } else {
                None
            },
        )),
        AttributeType::Bool => AttributeValue::Bool(RwSignal::new(Default::default())),
        AttributeType::File => AttributeValue::File(ValidateData::new_local(
            Default::default(),
            get_file_validators(&attribute),
        )),
        AttributeType::FileList => AttributeValue::FileList(ValidateData::new_local(
            Default::default(),
            get_file_list_validators(attribute),
        )),
    }
}

fn get_value(attribute: &Attribute, value: Value) -> AttributeValue {
    let validators = get_string_validators(attribute);
    match attribute.r#type {
        AttributeType::String => {
            let value = value
                .as_str()
                .map(|value| value.to_string())
                .unwrap_or_default();
            AttributeValue::String(ValidateData::new(value.into(), validators))
        }
        AttributeType::StringList => {
            let value: Vec<(Key, RwSignal<SharedString>)> = value
                .as_array()
                .map(|value| {
                    value
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(|value| {
                                    (gen_id().into(), RwSignal::new(value.to_string().into()))
                                })
                                .unwrap_or_else(|| {
                                    (gen_id().into(), RwSignal::new(Default::default()))
                                })
                        })
                        .collect()
                })
                .unwrap_or_default();
            AttributeValue::StringList(ValidateData::new(
                value.into(),
                get_string_list_validators(attribute),
            ))
        }
        AttributeType::LongString => {
            let value = value
                .as_str()
                .map(|value| value.to_string())
                .unwrap_or_default();
            AttributeValue::LongString(ValidateData::new(value.into(), validators))
        }
        // AttributeType::RichText => {
        //     let value = value
        //         .as_str()
        //         .map(|value| value.to_string())
        //         .unwrap_or_default();
        //     AttributeValue::RichText(ValidateData::new(
        //         value.into(),
        //         if attribute.required && AttributeType::Bool != attribute.r#type {
        //             Some(Validators::new().add(
        //                 crate::components::rich_text::RequiredValidator::new(format!(
        //                     "请输入{}",
        //                     attribute.name
        //                 )),
        //             ))
        //         } else {
        //             None
        //         },
        //     ))
        // }
        AttributeType::Code { .. } => {
            let value = value
                .as_str()
                .map(|value| value.to_string())
                .unwrap_or_default();
            AttributeValue::Code(ValidateData::new(value.into(), validators))
        }
        AttributeType::Password => {
            let value = value
                .as_str()
                .map(|value| value.to_string())
                .unwrap_or_default();
            AttributeValue::Password(ValidateData::new(value.into(), validators))
        }
        AttributeType::Enum { .. } => {
            let value = value
                .as_str()
                .map(|value| SharedString::from(value.to_string()));
            AttributeValue::Enum(ValidateData::new(
                value,
                if attribute.required {
                    Some(
                        Validators::new()
                            .add(RequiredValidator::new(format!("请选择{}", attribute.name))),
                    )
                } else {
                    None
                },
            ))
        }
        AttributeType::EnumList { .. } => {
            let value: Vec<_> = value
                .as_array()
                .map(|value| {
                    value
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(|value| SharedString::from(value.to_string()))
                        })
                        .filter_map(|v| v)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            AttributeValue::EnumList(ValidateData::new(
                value.into(),
                if attribute.required {
                    Some(
                        Validators::new()
                            .add(RequiredValidator::new(format!("请选择{}", attribute.name))),
                    )
                } else {
                    None
                },
            ))
        }
        AttributeType::Bool => {
            let value = value.as_bool().unwrap_or_default();
            AttributeValue::Bool(RwSignal::new(value))
        }
        AttributeType::File => {
            let value = match value {
                Value::Null => None,
                Value::Object(map) => {
                    let key = map
                        .get("key")
                        .unwrap()
                        .as_str()
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                    let name = map
                        .get("name")
                        .unwrap()
                        .as_str()
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                    let size = map.get("size").unwrap().as_f64().unwrap();
                    let mime_type = map
                        .get("mime_type")
                        .unwrap()
                        .as_str()
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                    Some(Resource::Remote(ResourceMetadata {
                        key: key,
                        name: name,
                        size: size,
                        mime_type: mime_type,
                    }))
                }
                _ => unreachable!(),
            };
            AttributeValue::File(ValidateData::new_local(
                value.into(),
                get_file_validators(&attribute),
            ))
        }
        AttributeType::FileList => {
            let value: Vec<(Key, RwSignal<Resource, LocalStorage>, ())> = value
                .as_array()
                .map(|value| {
                    value
                        .iter()
                        .map(|value| {
                            value
                                .as_object()
                                .map(|map| {
                                    let key = map
                                        .get("key")
                                        .unwrap()
                                        .as_str()
                                        .map(|value| value.to_string())
                                        .unwrap_or_default();
                                    let name = map
                                        .get("name")
                                        .unwrap()
                                        .as_str()
                                        .map(|value| value.to_string())
                                        .unwrap_or_default();
                                    let size = map.get("size").unwrap().as_f64().unwrap();
                                    let mime_type = map
                                        .get("mime_type")
                                        .unwrap()
                                        .as_str()
                                        .map(|value| value.to_string())
                                        .unwrap_or_default();
                                    let file = Resource::Remote(ResourceMetadata {
                                        key: key,
                                        name: name,
                                        size: size,
                                        mime_type: mime_type,
                                    });
                                    (gen_id().into(), RwSignal::new_local(file), ())
                                })
                                .unwrap()
                        })
                        .collect()
                })
                .unwrap_or_default();
            AttributeValue::FileList(ValidateData::new_local(
                value.into(),
                get_file_list_validators(attribute),
            ))
        }
    }
}

pub fn get_configuration_schema<'a>(
    extension_list: &'a [Extension],
    extension_id: &str,
) -> Option<&'a Vec<Attribute>> {
    return extension_list
        .iter()
        .find(|extension| extension.id == extension_id)
        .map(|extension| &extension.configuration_schema);
}

pub fn get_parameter_schema<'a>(
    operations: &'a [Operation],
    operation_id: &str,
) -> Option<&'a Vec<Attribute>> {
    return operations
        .iter()
        .find(|operation| operation.id == operation_id)
        .map(|operation| &operation.parameter_schema);
}

pub fn wrap_content(content: DocumentFragment) -> impl IntoView {
    let container = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    container.append_child(&content).unwrap();
    let inner_html = container.inner_html();
    return view! {
        <div inner_html={inner_html} class="rich-text" style="padding-left:0.25em;padding-right:0.25em;"/>
    };
}
