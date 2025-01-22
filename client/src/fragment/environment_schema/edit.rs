use crate::components::button::Button;
use crate::components::input::BindingInput;
use crate::components::required::Required;
use crate::components::selection::BindingSelection;
use crate::components::table::ArcRowRenderer;
use crate::components::table::Column;
use crate::components::table::Table;
use crate::components::validate_wrapper::ValidateData;
use crate::components::ArcRenderer;
use crate::sdk;
use crate::utils;
use crate::utils::binding::Binding;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::LightString;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::environment_schema::save_environment_schema::SaveEnvironmentSchemaApi;
use sdk::environment_schema::save_environment_schema::SaveEnvironmentSchemaReq;
use sdk::extension::query_extension::QueryExtensionApi;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::Extension;
use std::ops::Deref;
use tihu::Id;
use tihu::PrimaryKey;
use yew::prelude::*;
use yew::virtual_dom::Key;

type ExtensionSelection = BindingSelection<(LightString, String)>;

#[derive(Clone, PartialEq, Debug)]
pub struct SchemaResource {
    id: Option<Id>,
    name: ValidateData<LightString>,
    extension_id: ValidateData<Option<LightString>>,
}

#[derive(Clone)]
struct EditForm {
    name: ValidateData<LightString>,
    resource_list: UseStateHandle<Vec<(Key, Binding<SchemaResource>)>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Option<Id>,
    #[prop_or_default]
    pub onsave: Option<Callback<PrimaryKey>>,
}

#[function_component]
pub fn EnvironmentSchemaEdit(props: &Props) -> Html {
    let is_saving: UseStateHandle<bool> = use_state(|| false);
    let err_msg: UseStateHandle<Option<LightString>> = use_state(|| None);
    let extension_list: UseStateHandle<Vec<Extension>> = use_state(|| Default::default());
    let edit_form = EditForm {
        name: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请输入环境规格名称"))),
        ),
        resource_list: use_state(|| Default::default()),
    };
    let id = props.id;
    let edit_form_clone = edit_form.clone();
    let extension_list_clone = extension_list.clone();
    use_effect_with(id, move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            query_extension_list(&extension_list_clone).await.ok();
        });
        if let Some(id) = id {
            wasm_bindgen_futures::spawn_local(async move {
                read_environment_schema_detail(&edit_form_clone.clone(), id)
                    .await
                    .ok();
            });
        }
        || ()
    });
    let err_msg_clone = err_msg.clone();
    let clear_err_msg = Callback::from(move |_: ()| {
        err_msg_clone.set(None);
    });
    let edit_form_clone = edit_form.clone();
    let is_saving_clone = is_saving.clone();
    let err_msg_clone = err_msg.clone();
    let onsave_clone = props.onsave.clone();
    let on_save = Callback::from(move |_| {
        let edit_form: EditForm = edit_form_clone.clone();
        let is_saving = is_saving_clone.clone();
        let err_msg = err_msg_clone.clone();
        let onsave = onsave_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            save_environment_schema(id, &edit_form, is_saving, &err_msg, &onsave)
                .await
                .ok();
        });
    });

    let resource_list_clone1 = edit_form.resource_list.clone();
    let resource_list_clone2 = edit_form.resource_list.clone();
    let clear_err_msg_clone = clear_err_msg.clone();
    let columns: Vec<Column<Binding<SchemaResource>>> = vec![
        Column {
            key: "name".into(),
            head: ArcRenderer::from(move |_: &'_ ()| {
                html! { "规格名称" }
            }),
            row: ArcRowRenderer::from(move |item: &Binding<SchemaResource>, _index: usize| {
                let clear_err_msg = clear_err_msg_clone.clone();
                html! {
                    item.view(move |item: UseStateHandle<SchemaResource>| {
                        let clear_err_msg = clear_err_msg.clone();
                        item.name.view(move |name: UseStateHandle<LightString>, validator: Callback<LightString>| {
                            html! {
                                <BindingInput value={name} placeholder={"规格名称"} onupdate={validator} onfocus={clear_err_msg.clone()}/>
                            }
                        })
                    })
                }
            }),
            head_style: None,
            data_style: None,
        },
        Column {
            key: "extension".into(),
            head: ArcRenderer::from(move |_: &'_ ()| {
                html! { "资源类型" }
            }),
            row: ArcRowRenderer::from(move |item: &Binding<SchemaResource>, _index: usize| {
                let extension_list = extension_list.clone();
                html! {
                    item.view(move |item: UseStateHandle<SchemaResource>| {
                        let extension_list = extension_list.clone();
                        item.extension_id.view(move |extension_id: UseStateHandle<Option<LightString>>, validator: Callback<Option<LightString>>| {
                            let extension_list: Vec<_> = extension_list.iter().map(|item|(item.id.clone().into(), item.name.clone())).collect();
                            let onchange = validator.reform(|option: Option<(LightString, String)>| {
                                option.map(|option| option.0)
                            });
                            html! {
                                <ExtensionSelection value={extension_id.clone()} options={extension_list} onchange={onchange}/>
                            }
                        })
                    })
                }
            }),
            head_style: None,
            data_style: None,
        },
        Column {
            key: "operation".into(),
            head: ArcRenderer::from(move |_: &'_ ()| {
                html! { "操作" }
            }),
            row: ArcRowRenderer::from(move |_attr: &Binding<SchemaResource>, index: usize| {
                let resource_list = resource_list_clone1.clone();
                let on_remove = Callback::from(move |_| {
                    let mut new_resource_list = resource_list.deref().clone();
                    new_resource_list.remove(index);
                    resource_list.set(new_resource_list);
                });
                html! {
                    <Button onclick={on_remove} style="margin-left:0.5em;">{"移除"}</Button>
                }
            }),
            head_style: None,
            data_style: Some((|_index: usize| AttrValue::from("vertical-align: top;")).into()),
        },
    ];
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"环境规格名称："}</td>
                    <td>
                        {
                            edit_form.name.view(move |name: UseStateHandle<LightString>, validator: Callback<LightString>| {
                                html! {
                                    <BindingInput value={name} onupdate={validator}/>
                                }
                            })
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"资源规格："}</td>
                    <td>
                        {
                            {
                                let on_add = Callback::from(move |_| {
                                    let mut resource_list_clone = resource_list_clone2.deref().clone();
                                    resource_list_clone.push((
                                        utils::gen_id().into(),
                                        Binding::new(SchemaResource {
                                            id: None,
                                            name: init_resource_name(Default::default()),
                                            extension_id: init_extension_id(None),
                                        }),
                                    ));
                                    resource_list_clone2.set(resource_list_clone);
                                });
                                html! {
                                    <>
                                        <Table<Binding<SchemaResource>> list={edit_form.resource_list.deref().clone()} columns={columns} />
                                        <div style="margin-top:0.5em">
                                            <Button onclick={on_add}>{"添加"}</Button>
                                        </div>
                                    </>
                                }
                            }
                        }
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td style="padding-top:0.5em">
                        <Button disabled={*is_saving} onclick={on_save}>{"保存"}</Button>
                        {
                            match err_msg.as_ref() {
                                Some(err_msg) => {
                                    html!{
                                        <span class="middle" style="color:red;margin-left: 0.5em;">{err_msg}</span>
                                    }
                                },
                                None => html!{}
                            }
                        }
                    </td>
                </tr>
            </table>
        </div>
    }
}

async fn query_extension_list(
    extension_list: &UseStateHandle<Vec<Extension>>,
) -> Result<(), LightString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result);
    return Ok(());
}

fn init_resource_name(value: LightString) -> ValidateData<LightString> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请输入资源名称"))),
    )
}

fn init_extension_id(value: Option<LightString>) -> ValidateData<Option<LightString>> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请选择扩展"))),
    )
}

async fn read_environment_schema_detail(edit_form: &EditForm, id: Id) -> Result<(), LightString> {
    let params = ReadEnvironmentSchemaReq { id: id };
    let environment_schema = ReadEnvironmentSchemaApi.call(&params).await?;
    edit_form.name.set(environment_schema.name.into());
    edit_form.resource_list.set(
        environment_schema
            .resource_list
            .into_iter()
            .map(|schema_resource| {
                (
                    utils::gen_id().into(),
                    Binding::new(SchemaResource {
                        id: Some(schema_resource.id),
                        name: init_resource_name(schema_resource.name.into()),
                        extension_id: init_extension_id(Some(schema_resource.extension_id.into())),
                    }),
                )
            })
            .collect(),
    );
    return Ok(());
}

fn chk_form_err(edit_form: &EditForm) -> Vec<LightString> {
    let mut err_msgs: Vec<LightString> = Vec::new();
    if let Err(error) = edit_form.name.validate(true) {
        err_msgs.push(error);
    }
    for (_, resource) in edit_form.resource_list.iter() {
        if let Err(error) = resource.get().name.validate(true) {
            err_msgs.push(error);
        }
        if let Err(error) = resource.get().extension_id.validate(true) {
            err_msgs.push(error);
        }
    }
    return err_msgs;
}

fn collect_resource_list(
    edit_form: &EditForm,
) -> Vec<sdk::environment_schema::save_environment_schema::SchemaResource> {
    let mut resource_list: Vec<_> = Vec::new();
    for (_, schema_resource_opt) in edit_form.resource_list.deref().iter() {
        let schema_resource = schema_resource_opt.get();
        resource_list.push(
            sdk::environment_schema::save_environment_schema::SchemaResource {
                id: schema_resource.id,
                name: schema_resource.name.get().to_string(),
                extension_id: schema_resource
                    .extension_id
                    .get()
                    .map(|extension_id| extension_id.to_string())
                    .unwrap(),
            },
        );
    }
    return resource_list;
}

async fn save_environment_schema(
    id: Option<Id>,
    edit_form: &EditForm,
    is_saving: UseStateHandle<bool>,
    err_msg: &UseStateHandle<Option<LightString>>,
    onsave: &Option<Callback<PrimaryKey>>,
) -> Result<(), LightString> {
    let err_msgs = chk_form_err(edit_form);
    if let Some(first) = err_msgs.first() {
        err_msg.set(Some(first.clone()));
        return Err(first.clone());
    }
    let resource_list = collect_resource_list(edit_form);
    let params = SaveEnvironmentSchemaReq {
        id: id,
        name: edit_form.name.get().to_string(),
        resource_list: resource_list,
    };
    let ret = SaveEnvironmentSchemaApi
        .lock_handler(is_saving)
        .call(&params)
        .await;
    match ret {
        Err(err) => {
            log::error!("{}", err);
            err_msg.set(Some(err));
        }
        Ok(pri_key) => {
            match onsave {
                Some(onsave) => {
                    onsave.emit(pri_key);
                }
                None => (),
            }
            utils::success(LightString::from("保存成功"));
        }
    }
    return Ok(());
}
