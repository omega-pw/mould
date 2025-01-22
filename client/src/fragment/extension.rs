use crate::components::button::Button;
use crate::components::checkbox::BindingCheckbox;
use crate::components::checkbox_group::BindingCheckboxGroup;
use crate::components::file_upload::BindingFileUpload;
use crate::components::files_upload::BindingFilesUpload;
use crate::components::input::BindingInput;
use crate::components::input::Input;
use crate::components::monaco_editor::BindingMonacoEditor;
use crate::components::r#if::If;
use crate::components::radio_group::BindingRadioGroup;
use crate::components::required::Required;
// use crate::components::rich_text::render_rich_rext;
// use crate::components::rich_text::BindingRichText;
use crate::components::textarea::BindingTextarea;
use crate::components::validate_wrapper::ValidateData;
use crate::components::File;
use crate::sdk;
use crate::utils::binding::Binding;
use crate::utils::gen_id;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::LightString;
// use js_sys::JSON;
use sdk::extension::Attribute;
use sdk::extension::AttributeType;
use sdk::extension::Extension;
use sdk::extension::Operation;
use serde_json::Value;
use std::ops::Deref;
// use wasm_bindgen::prelude::*;
use web_sys::DocumentFragment;
use yew::prelude::*;
use yew::virtual_dom::Key;

type EnumRadioGroup = BindingRadioGroup<(LightString, String)>;
type EnumCheckboxGroup = BindingCheckboxGroup<(LightString, String)>;

#[derive(Clone, PartialEq, Debug)]
pub enum AttributeValue {
    String(ValidateData<LightString>),
    StringList(ValidateData<Vec<(Key, LightString)>>),
    LongString(ValidateData<LightString>),
    // RichText(ValidateData<JsValue>),
    Code(ValidateData<LightString>),
    Password(ValidateData<LightString>),
    Enum(ValidateData<LightString>),
    EnumList(ValidateData<Vec<LightString>>),
    Bool(Binding<bool>),
    File(ValidateData<Option<File>>),
    FileList(ValidateData<Vec<(Key, File, ())>>),
}

impl AttributeValue {
    pub fn validate(&self, update_view: bool) -> Result<(), LightString> {
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

pub fn config_view(attributes: &[(Key, Attribute, AttributeValue)]) -> Html {
    html! {
        for attributes.iter().map(|(key, attribute, value)| {
            match &attribute.r#type {
                AttributeType::String => {
                    let title = if let Some(description) = attribute.description.as_ref() {
                        format!("{}({})", attribute.name, description)
                    } else {
                        format!("{}", attribute.name)
                    };
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::String(value) => {
                                            value.view(move |value, validator| {
                                                html! {
                                                    <BindingInput value={value} onupdate={validator}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::StringList(value_list) => {
                                            value_list.view(move |value_list: UseStateHandle<Vec<(Key, LightString)>>, validator: Callback<Vec<(Key, LightString)>>| {
                                                let value_list_clone = value_list.clone();
                                                let validator_clone = validator.clone();
                                                html! {
                                                    <div>
                                                        {
                                                            for value_list.iter().enumerate().map(|(index, (key, value))| {
                                                                let value_list = value_list_clone.clone();
                                                                let validator = validator_clone.clone();
                                                                let on_update = Callback::from(move |new_value: AttrValue| {
                                                                    let mut new_items = value_list.deref().clone();
                                                                    new_items[index].1 = new_value;
                                                                    value_list.set(new_items.clone());
                                                                    validator.emit(new_items);
                                                                });
                                                                let value_list = value_list_clone.clone();
                                                                let validator = validator_clone.clone();
                                                                let on_remove = Callback::from(move |_| {
                                                                    let mut new_items = value_list.deref().clone();
                                                                    new_items.remove(index);
                                                                    value_list.set(new_items.clone());
                                                                    validator.emit(new_items);
                                                                });
                                                                html! {
                                                                    <div key={key.clone()}>
                                                                        <Input value={value.clone()} onupdate={on_update}/>
                                                                        <Button onclick={on_remove}>{"Remove"}</Button>
                                                                    </div>
                                                                }
                                                            })
                                                        }
                                                        <Button onclick={Callback::from(move |_| {
                                                            let mut new_items = value_list_clone.deref().clone();
                                                            new_items.push((gen_id().into(), LightString::from("")));
                                                            value_list_clone.set(new_items.clone());
                                                            validator_clone.emit(new_items);
                                                        })}>{"Add"}</Button>
                                                    </div>
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::LongString(value) => {
                                            value.view(move |value, validator| {
                                                html! {
                                                    <BindingTextarea value={value} onupdate={validator} style="width:100%;"/>
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                //     html! {
                //         <div key={key.clone()}>
                //             <div>
                //                 <If condition={attribute.required}><Required/></If>
                //                 { title }
                //             </div>
                //             <div>
                //                 {
                //                     match value {
                //                         AttributeValue::RichText(value) => {
                //                             value.view(move |value, validator: Callback<JsValue>| {
                //                                 html! {
                //                                     <BindingRichText value={value} onchange={validator} style="border: 1px solid rgba(0, 0, 0, 0.2);padding: 0.25em 0;min-height: 8em;"/>
                //                                 }
                //                             })
                //                         },
                //                         _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::Code(value) => {
                                            let language = language.clone();
                                            value.view(move |value, validator: Callback<LightString>| {
                                                let language = LightString::from(language.clone());
                                                html! {
                                                    <BindingMonacoEditor value={value} language={language} width="100%" height="16em" onchange={validator}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::Password(value) => {
                                            value.view(move |value, validator| {
                                                html! {
                                                    <BindingInput r#type="password" disable_trim={true} value={value} onupdate={validator}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::Enum(value) => {
                                            let options = options.clone();
                                            value.view(move |value, validator: Callback<LightString>| {
                                                let options: Vec<_> = options.iter().map(|option| {
                                                    (option.value.clone().into(), option.label.clone())
                                                }).collect();
                                                let onchange = validator.reform(|(value, _label): (LightString, String)| {
                                                    value
                                                });
                                                html! {
                                                    <EnumRadioGroup value={value} options={options} onchange={onchange}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::EnumList(values) => {
                                            let options = options.clone();
                                            values.view(move |values, validator: Callback<Vec<LightString>>| {
                                                let options: Vec<_> = options.iter().map(|option| {
                                                    (option.value.clone().into(), option.label.clone())
                                                }).collect();
                                                html! {
                                                    <EnumCheckboxGroup value={values} options={options} onchange={validator}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
                                    }
                                }
                            </div>
                        </div>
                    }
                },
                AttributeType::Bool => {
                    let name = attribute.name.clone();
                    let description = if let Some(description) = attribute.description.as_ref() {
                        format!("({})", description)
                    } else {
                        String::from("")
                    };
                    html! {
                        <div key={key.clone()}>
                            {
                                match value {
                                    AttributeValue::Bool(value) => {
                                        value.view(move |value| {
                                            html! {
                                                <>
                                                    <BindingCheckbox value={value} label={name.clone()} />
                                                    { description.clone() }
                                                </>
                                            }
                                        })
                                    },
                                    _ => html!{}
                                }
                            }
                        </div>
                    }
                }
                AttributeType::File => {
                    let title = if let Some(description) = attribute.description.as_ref() {
                        format!("{}({})", attribute.name, description)
                    } else {
                        format!("{}", attribute.name)
                    };
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::File(file) => {
                                            file.view(move |file, validator| {
                                                html! {
                                                    <BindingFileUpload file={file} onchange={validator}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
                                    }
                                }
                            </div>
                        </div>
                    }
                }
                AttributeType::FileList => {
                    let title = if let Some(description) = attribute.description.as_ref() {
                        format!("{}({})", attribute.name, description)
                    } else {
                        format!("{}", attribute.name)
                    };
                    html! {
                        <div key={key.clone()}>
                            <div>
                                <If condition={attribute.required}><Required/></If>
                                { title }
                            </div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::FileList(files) => {
                                            files.view(move |files, validator| {
                                                html! {
                                                    <BindingFilesUpload<()> files={files} onchange={validator}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
                                    }
                                }
                            </div>
                        </div>
                    }
                }
            }
        })
    }
}

pub fn config_detail_view(attributes: &[(Key, Attribute, AttributeValue)]) -> Html {
    html! {
        for attributes.iter().map(|(key, attribute, value)| {
            match &attribute.r#type {
                AttributeType::String => {
                    let title = if let Some(description) = attribute.description.as_ref() {
                        format!("{}({})", attribute.name, description)
                    } else {
                        format!("{}", attribute.name)
                    };
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::String(value) => {
                                            value.view(move |value: UseStateHandle<LightString>, _validator| {
                                                html! { value.deref().clone() }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::StringList(value) => {
                                            value.view(move |value_list: UseStateHandle<Vec<(Key, LightString)>>, _validator| {
                                                html! {
                                                    for value_list.iter().map(|(key, value)| {
                                                        html! {
                                                            <div key={key.clone()}>{value}</div>
                                                        }
                                                    })
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::LongString(value) => {
                                            value.view(move |value: UseStateHandle<LightString>, _validator| {
                                                html! { value.deref().clone() }
                                            })
                                        },
                                        _ => html!{}
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
                //     html! {
                //         <div key={key.clone()}>
                //             <div>{ title }</div>
                //             <div>
                //                 {
                //                     match value {
                //                         AttributeValue::RichText(value) => {
                //                             value.view(move |value: UseStateHandle<JsValue>, _validator: Callback<JsValue>| {
                //                                 let content = render_rich_rext(&value).unwrap();
                //                                 wrap_content(content)
                //                             })
                //                         },
                //                         _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::Code(value) => {
                                            let language = language.clone();
                                            value.view(move |value, validator: Callback<LightString>| {
                                                let language = LightString::from(language.clone());
                                                html! {
                                                    <BindingMonacoEditor value={value} language={language} readonly={true} width="100%" height="16em" onchange={validator}/>
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::Password(value) => {
                                            value.view(move |value: UseStateHandle<LightString>, _validator| {
                                                let len = value.deref().chars().count();
                                                Html::from("*".repeat(len))
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::Enum(value) => {
                                            let options = options.clone();
                                            value.view(move |value: UseStateHandle<LightString>, _validator: Callback<LightString>| {
                                                let label = options.iter().find(|option| {
                                                    value.deref() == &option.value
                                                }).map(|option| option.label.clone()).unwrap_or_default();
                                                html! { label }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::EnumList(values) => {
                                            let options = options.clone();
                                            values.view(move |values: UseStateHandle<Vec<LightString>>, _validator: Callback<Vec<LightString>>| {
                                                let labels: Vec<&str> = options.iter().filter(|option| {
                                                    values.iter().any(|value| value == &option.value)
                                                }).map(|option| option.label.as_ref()).collect();
                                                let labels = labels.join(",");
                                                html! { labels }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::Bool(value) => {
                                            value.view(move |value: UseStateHandle<bool>| {
                                                let value = if *value {
                                                    "是"
                                                } else {
                                                    "否"
                                                };
                                                html! { value }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::File(file) => {
                                            file.view(move |file: UseStateHandle<Option<File>>, _validator| {
                                                if let Some(file) = file.as_ref() {
                                                    match file {
                                                        File::Remote { key, name, .. } => {
                                                            let url = format!("/blob/{}", key);
                                                            html! {
                                                                <a href={url} target="_blank">{name}</a>
                                                            }
                                                        }
                                                        File::Local(hashing_file) => {
                                                            html! { hashing_file.file.name() }
                                                        }
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            })
                                        },
                                        _ => html!{}
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
                    html! {
                        <div key={key.clone()}>
                            <div>{ title }</div>
                            <div>
                                {
                                    match value {
                                        AttributeValue::FileList(files) => {
                                            files.view(move |files: UseStateHandle<Vec<(Key, File, ())>>, _validator| {
                                                html! {
                                                    for files.iter().map(|(_key, file, _)| {
                                                        match file {
                                                            File::Remote { key, name, .. } => {
                                                                let url = format!("/blob/{}", key);
                                                                html! {
                                                                    <a href={url} target="_blank">{name}</a>
                                                                }
                                                            }
                                                            File::Local(hashing_file) => {
                                                                html! { hashing_file.file.name() }
                                                            }
                                                        }
                                                    })
                                                }
                                            })
                                        },
                                        _ => html!{}
                                    }
                                }
                            </div>
                        </div>
                    }
                },
            }
        })
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
                    .map(|(_, value)| Value::String(value.to_string()))
                    .collect(),
            ),
            AttributeValue::LongString(value) => Value::String(value.get().to_string()),
            // AttributeValue::RichText(value) => {
            //     Value::String(JSON::stringify(&value.get()).unwrap().as_string().unwrap())
            // }
            AttributeValue::Code(value) => Value::String(value.get().to_string()),
            AttributeValue::Password(value) => Value::String(value.get().to_string()),
            AttributeValue::Enum(value) => Value::String(value.get().to_string()),
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
                        File::Remote {
                            key,
                            name,
                            size,
                            mime_type,
                        } => {
                            let mut map = serde_json::Map::new();
                            map.insert(String::from("key"), Value::String(key));
                            map.insert(String::from("name"), Value::String(name));
                            map.insert(String::from("size"), Value::from(size));
                            map.insert(String::from("mime_type"), Value::String(mime_type));
                            Value::Object(map)
                        }
                        File::Local(_) => {
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
                    .map(|(_, file, _)| match file {
                        File::Remote {
                            key,
                            name,
                            size,
                            mime_type,
                        } => {
                            let mut map = serde_json::Map::new();
                            map.insert(String::from("key"), Value::String(key));
                            map.insert(String::from("name"), Value::String(name));
                            map.insert(String::from("size"), Value::from(size));
                            map.insert(String::from("mime_type"), Value::String(mime_type));
                            Value::Object(map)
                        }
                        File::Local(_) => {
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

fn get_string_validators(attribute: &Attribute) -> Option<Validators<LightString>> {
    return if attribute.required && AttributeType::Bool != attribute.r#type {
        Some(Validators::new().add(RequiredValidator::new(format!("请输入{}", attribute.name))))
    } else {
        None
    };
}

fn get_string_list_validators(
    attribute: &Attribute,
) -> Option<Validators<Vec<(Key, LightString)>>> {
    return if attribute.required && AttributeType::Bool != attribute.r#type {
        Some(Validators::new().add(RequiredValidator::new(format!("请输入{}", attribute.name))))
    } else {
        None
    };
}

fn get_file_list_validators(attribute: &Attribute) -> Option<Validators<Vec<(Key, File, ())>>> {
    return if attribute.required && AttributeType::Bool != attribute.r#type {
        Some(Validators::new().add(RequiredValidator::new(format!(
            "请上传文件{}",
            attribute.name
        ))))
    } else {
        None
    };
}

fn get_file_validators(attribute: &Attribute) -> Option<Validators<Option<File>>> {
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
        AttributeType::Bool => AttributeValue::Bool(Binding::new(Default::default())),
        AttributeType::File => AttributeValue::File(ValidateData::new(
            Default::default(),
            get_file_validators(&attribute),
        )),
        AttributeType::FileList => AttributeValue::FileList(ValidateData::new(
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
            let value: Vec<(Key, LightString)> = value
                .as_array()
                .map(|value| {
                    value
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(|value| (gen_id().into(), value.to_string().into()))
                                .unwrap_or_else(|| (gen_id().into(), Default::default()))
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
                .map(|value| value.to_string())
                .unwrap_or_default();
            AttributeValue::Enum(ValidateData::new(
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
        AttributeType::EnumList { .. } => {
            let value: Vec<_> = value
                .as_array()
                .map(|value| {
                    value
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(|value| LightString::from(value.to_string()))
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
            AttributeValue::Bool(Binding::new(value))
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
                    Some(File::Remote {
                        key: key,
                        name: name,
                        size: size,
                        mime_type: mime_type,
                    })
                }
                _ => unreachable!(),
            };
            AttributeValue::File(ValidateData::new(
                value.into(),
                get_file_validators(&attribute),
            ))
        }
        AttributeType::FileList => {
            let value: Vec<(Key, File, ())> = value
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
                                    let file = File::Remote {
                                        key: key,
                                        name: name,
                                        size: size,
                                        mime_type: mime_type,
                                    };
                                    (gen_id().into(), file, ())
                                })
                                .unwrap()
                        })
                        .collect()
                })
                .unwrap_or_default();
            AttributeValue::FileList(ValidateData::new(
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

pub fn wrap_content(content: DocumentFragment) -> Html {
    let container = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    container.set_attribute("class", "rich-text").unwrap();
    container
        .set_attribute("style", "padding-left:0.25em;padding-right:0.25em;")
        .unwrap();
    container.append_child(&content).unwrap();
    return html! {
        { Html::VRef(container.into()) }
    };
}
