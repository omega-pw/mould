use crate::components::button::Button;
use crate::components::input::Input;
use crate::components::required::Required;
use crate::components::selection::Selection;
use crate::components::table::ArcRowRenderer;
use crate::components::table::Column;
use crate::components::table::Table;
use crate::components::validate_wrapper::ValidateData;
use crate::components::validate_wrapper::ValidateWrapper;
use crate::components::ArcRenderer;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::Key;
use crate::SharedString;
use leptos::prelude::*;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::environment_schema::save_environment_schema::SaveEnvironmentSchemaApi;
use sdk::environment_schema::save_environment_schema::SaveEnvironmentSchemaReq;
use sdk::extension::query_extension::QueryExtensionApi;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::Extension;
use tihu::Id;
use tihu::PrimaryKey;

#[derive(Clone)]
pub struct SchemaResource {
    id: Option<Id>,
    name: ValidateData<SharedString>,
    extension_id: ValidateData<Option<SharedString>>,
}

#[derive(Clone)]
struct EditForm {
    name: ValidateData<SharedString>,
    resource_list: RwSignal<Vec<(Key, SchemaResource)>>,
}

#[component]
pub fn EnvironmentSchemaEdit(
    #[prop(into, default = None)] id: Option<Id>,
    #[prop(into, default = None)] onsave: Option<UnsyncCallback<PrimaryKey>>,
) -> impl IntoView {
    let is_saving: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let extension_list: RwSignal<Vec<Extension>> = RwSignal::new(Default::default());
    let edit_form = EditForm {
        name: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请输入环境规格名称"))),
        ),
        resource_list: RwSignal::new(Default::default()),
    };
    let edit_form_clone = edit_form.clone();
    let extension_list_clone = extension_list.clone();
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
    let err_msg_clone = err_msg.clone();
    let clear_err_msg = UnsyncCallback::new(move |_: ()| {
        err_msg_clone.set(None);
    });
    let on_save = {
        let edit_form_clone = edit_form.clone();
        let is_saving_clone = is_saving.clone();
        let err_msg_clone = err_msg.clone();
        let onsave_clone = onsave.clone();
        UnsyncCallback::new(move |_| {
            let edit_form: EditForm = edit_form_clone.clone();
            let is_saving = is_saving_clone.clone();
            let err_msg = err_msg_clone.clone();
            let onsave = onsave_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                save_environment_schema(id, &edit_form, is_saving, &err_msg, &onsave)
                    .await
                    .ok();
            });
        })
    };

    let resource_list_clone1 = edit_form.resource_list.clone();
    let resource_list_clone2 = edit_form.resource_list.clone();
    let clear_err_msg_clone = clear_err_msg.clone();
    let columns: Vec<Column<SchemaResource>> = vec![
        Column {
            key: "name".into(),
            head: ArcRenderer::from(move |_: &'_ ()| "规格名称".into_any()),
            row: ArcRowRenderer::from(move |item: &SchemaResource, _index: usize| {
                let name = item.name.clone();
                view! {
                    <ValidateWrapper error={name.error()}>
                        <Input value={name.data()} placeholder={"规格名称"} onupdate={name.listener()} onfocus={clear_err_msg_clone.clone()}/>
                    </ValidateWrapper>
                }.into_any()
            }),
            head_style: None,
            data_style: None,
        },
        Column {
            key: "extension".into(),
            head: ArcRenderer::from(move |_: &'_ ()| "资源类型".into_any()),
            row: ArcRowRenderer::from(move |item: &SchemaResource, _index: usize| {
                let extension_list = {
                    let extension_list = extension_list.clone();
                    Signal::derive(move || {
                        extension_list
                            .read()
                            .iter()
                            .map(|item| (item.id.clone().into(), item.name.clone()))
                            .collect()
                    })
                };
                let onchange = {
                    let extension_id = item.extension_id.clone();
                    UnsyncCallback::new(move |_| {
                        extension_id.validate(true);
                    })
                };
                let extension_id = item.extension_id.clone();
                view! {
                    <ValidateWrapper error={extension_id.error()}>
                        <Selection value={extension_id.data()} options={extension_list} onchange={onchange}/>
                    </ValidateWrapper>
                }.into_any()
            }),
            head_style: None,
            data_style: None,
        },
        Column {
            key: "operation".into(),
            head: ArcRenderer::from(move |_: &'_ ()| "操作".into_any()),
            row: ArcRowRenderer::from(move |_attr: &SchemaResource, index: usize| {
                let resource_list = resource_list_clone1.clone();
                let on_remove = UnsyncCallback::new(move |_| {
                    resource_list.write().remove(index);
                });
                view! {
                    <Button onclick={on_remove} style={SharedString::from("margin-left:0.5em;")}>{"移除"}</Button>
                }
                .into_any()
            }),
            head_style: None,
            data_style: Some((|_index: usize| SharedString::from("vertical-align: top;")).into()),
        },
    ];
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"环境规格名称："}</td>
                    <td>
                        <ValidateWrapper error={edit_form.name.error()}>
                            <Input value={edit_form.name.data()} onupdate={edit_form.name.listener()}/>
                        </ValidateWrapper>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"资源规格："}</td>
                    <td>
                        {
                            {
                                let on_add = UnsyncCallback::new(move |_| {
                                    resource_list_clone2.write().push((
                                        utils::gen_id().into(),
                                        SchemaResource {
                                            id: None,
                                            name: init_resource_name(Default::default()),
                                            extension_id: init_extension_id(None),
                                        },
                                    ));
                                });
                                view! {
                                    <Table list={edit_form.resource_list} columns={columns} />
                                    <div style="margin-top:0.5em">
                                        <Button onclick={on_add}>{"添加"}</Button>
                                    </div>
                                }
                            }
                        }
                    </td>
                </tr>
                <tr>
                    <td></td>
                    <td style="padding-top:0.5em">
                        <Button disabled={is_saving} onclick={on_save}>{"保存"}</Button>
                        <Show
                            when={ let err_msg = err_msg.clone(); move || { err_msg.read().is_some() } }
                        >
                            <span class="middle" style="color:red;margin-left: 0.5em;">{err_msg}</span>
                        </Show>
                    </td>
                </tr>
            </table>
        </div>
    }
}

async fn query_extension_list(
    extension_list: &RwSignal<Vec<Extension>>,
) -> Result<(), SharedString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result);
    return Ok(());
}

fn init_resource_name(value: SharedString) -> ValidateData<SharedString> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请输入资源名称"))),
    )
}

fn init_extension_id(value: Option<SharedString>) -> ValidateData<Option<SharedString>> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请选择扩展"))),
    )
}

async fn read_environment_schema_detail(edit_form: &EditForm, id: Id) -> Result<(), SharedString> {
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
                    SchemaResource {
                        id: Some(schema_resource.id),
                        name: init_resource_name(schema_resource.name.into()),
                        extension_id: init_extension_id(Some(schema_resource.extension_id.into())),
                    },
                )
            })
            .collect(),
    );
    return Ok(());
}

fn chk_form_err(edit_form: &EditForm) -> Vec<SharedString> {
    let mut err_msgs: Vec<SharedString> = Vec::new();
    if let Err(error) = edit_form.name.validate(true) {
        err_msgs.push(error);
    }
    for (_, resource) in edit_form.resource_list.read().iter() {
        if let Err(error) = resource.name.validate(true) {
            err_msgs.push(error);
        }
        if let Err(error) = resource.extension_id.validate(true) {
            err_msgs.push(error);
        }
    }
    return err_msgs;
}

fn collect_resource_list(
    edit_form: &EditForm,
) -> Vec<sdk::environment_schema::save_environment_schema::SchemaResource> {
    let mut resource_list: Vec<_> = Vec::new();
    for (_, schema_resource) in edit_form.resource_list.read().iter() {
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
    is_saving: RwSignal<bool>,
    err_msg: &RwSignal<Option<SharedString>>,
    onsave: &Option<UnsyncCallback<PrimaryKey>>,
) -> Result<(), SharedString> {
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
                    onsave.run(pri_key);
                }
                None => (),
            }
            utils::success(SharedString::from("保存成功"));
        }
    }
    return Ok(());
}
