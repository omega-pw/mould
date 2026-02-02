use super::super::extension::get_configuration_schema;
use super::super::extension::get_default_config;
use super::super::extension::parse_config;
use super::super::extension::serialize_config;
use super::super::extension::AttributeValue;
use super::super::extension::ConfigView;
use crate::components::button::Button;
use crate::components::input::Input;
use crate::components::required::Required;
use crate::components::selection::Selection;
use crate::components::uploading_files::upload_files;
use crate::components::validate_wrapper::ValidateData;
use crate::components::validate_wrapper::ValidateWrapper;
use crate::components::visable::Visable;
use crate::components::Resource;
use crate::sdk;
use crate::utils;
use crate::utils::gen_id;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::Key;
use crate::SharedString;
use leptos::prelude::*;
use sdk::environment::insert_environment::InsertEnvironmentApi;
use sdk::environment::insert_environment::InsertEnvironmentReq;
use sdk::environment::read_environment::Environment;
use sdk::environment::read_environment::ReadEnvironmentApi;
use sdk::environment::read_environment::ReadEnvironmentReq;
use sdk::environment::update_environment::UpdateEnvironmentApi;
use sdk::environment::update_environment::UpdateEnvironmentReq;
use sdk::environment_schema::query_environment_schema::QueryEnvironmentSchemaApi;
use sdk::environment_schema::query_environment_schema::QueryEnvironmentSchemaReq;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema as EnvironmentSchemaDetail;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::environment_schema::EnvironmentSchema;
use sdk::extension::query_extension::QueryExtensionApi;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::test_configuration::TestConfigurationApi;
use sdk::extension::test_configuration::TestConfigurationReq;
use sdk::extension::Attribute;
use sdk::extension::Extension;
use std::sync::Arc;
use std::sync::Mutex;
use tihu::Id;
use tihu::PrimaryKey;

#[derive(Clone)]
pub struct EnvironmentResource {
    id: Option<Id>,
    name: ValidateData<SharedString>,
    extension_configuration: Vec<(Key, Attribute, AttributeValue)>, //扩展配置
    test_error: RwSignal<Option<Result<(), SharedString>>>,
}

/**
 * 环境规格资源
 */
#[derive(Clone, Debug)]
pub struct EnvironmentSchemaResource {
    id: Id,
    extension_id: String,
    name: String,
    resource_list: RwSignal<Vec<(Key, EnvironmentResource)>>,
}

#[derive(Clone)]
struct EditForm {
    active_schema_resource_id: RwSignal<Option<Id>>,
    active_resource_key: RwSignal<Option<Key>>,
    environment_schema_id: ValidateData<Option<Id>>,
    name: ValidateData<SharedString>,
    schema_resource_list: RwSignal<Vec<(Key, EnvironmentSchemaResource)>>,
}

#[derive(Clone)]
struct EnvironmentEditState {
    is_saving: RwSignal<bool>,
    err_msg: RwSignal<Option<SharedString>>,
    environment_schema_list: RwSignal<Vec<EnvironmentSchema>>,
    environment_schema_detail: RwSignal<Option<EnvironmentSchemaDetail>>,
    extension_list: RwSignal<Vec<Extension>>,
    edit_form: EditForm,
}

#[component]
pub fn EnvironmentEdit(
    #[prop(into, default = None)] id: Option<Id>,
    #[prop(into, default = None)] onsave: Option<UnsyncCallback<PrimaryKey>>,
) -> impl IntoView {
    let is_saving: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let environment_schema_list: RwSignal<Vec<EnvironmentSchema>> = RwSignal::new(Vec::new());
    let environment_schema_detail: RwSignal<Option<EnvironmentSchemaDetail>> = RwSignal::new(None);
    let extension_list: RwSignal<Vec<Extension>> = RwSignal::new(Default::default());
    let edit_form = EditForm {
        active_schema_resource_id: RwSignal::new(Default::default()),
        active_resource_key: RwSignal::new(Default::default()),
        environment_schema_id: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请选择环境规格"))),
        ),
        name: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请输入环境名称"))),
        ),
        schema_resource_list: RwSignal::new(Vec::new()),
    };
    let environment_edit_state = EnvironmentEditState {
        is_saving: is_saving.clone(),
        err_msg: err_msg.clone(),
        environment_schema_list: environment_schema_list.clone(),
        environment_schema_detail: environment_schema_detail.clone(),
        extension_list: extension_list.clone(),
        edit_form: edit_form.clone(),
    };
    let environment_schema_id = edit_form.environment_schema_id.clone();
    let schema_resource_list = edit_form.schema_resource_list.clone();
    let schema_resource_list_clone = edit_form.schema_resource_list.clone();
    let edit_form_clone = edit_form.clone();
    let environment_schema_list_clone = environment_schema_list.clone();
    let extension_list_clone = extension_list.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let environment_schema_detail_clone2 = environment_schema_detail.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match query_environment_schema_list(&environment_schema_list_clone).await {
            Ok(environment_schema_list) => {
                if id.is_none() {
                    //新增场景，默认选择第一个环境规格
                    if let Some(environment_schema) = environment_schema_list.first() {
                        if let Ok(_) = handle_environment_schema_change(
                            environment_schema.id,
                            &environment_schema_detail_clone,
                            &schema_resource_list_clone,
                        )
                        .await
                        {
                            environment_schema_id.set(Some(environment_schema.id));
                        }
                    } else {
                        utils::error(SharedString::from("请先添加环境规格"));
                    }
                }
            }
            Err(_err) => {
                //
            }
        }
    });
    wasm_bindgen_futures::spawn_local(async move {
        match query_extension_list(&extension_list_clone).await {
            Ok(extension_list) => {
                if let Some(id) = id {
                    match read_environment_detail(&edit_form_clone.clone(), &extension_list, id)
                        .await
                    {
                        Ok(environment) => {
                            read_environment_schema_detail(
                                &environment_schema_detail_clone2,
                                environment.environment_schema_id,
                            )
                            .await
                            .ok();
                        }
                        Err(_err) => {
                            //
                        }
                    }
                }
            }
            Err(_err) => {
                //
            }
        }
    });
    let edit_form_clone = edit_form.clone();
    let is_saving_clone = is_saving.clone();
    let err_msg_clone = err_msg.clone();
    let on_save = UnsyncCallback::new(move |_| {
        let edit_form: EditForm = edit_form_clone.clone();
        let is_saving = is_saving_clone.clone();
        let err_msg = err_msg_clone.clone();
        let onsave = onsave.clone();
        wasm_bindgen_futures::spawn_local(async move {
            save_environment(id, &edit_form, is_saving, &err_msg, &onsave)
                .await
                .ok();
        });
    });
    let environment_schema_list = Signal::derive(move || {
        environment_schema_list
            .read()
            .iter()
            .map(|item| (item.id.clone().into(), item.name.clone()))
            .collect()
    });
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"环境名称："}</td>
                    <td>
                        <ValidateWrapper error={edit_form.name.error()}>
                            <Input value={edit_form.name.data()} onupdate={edit_form.name.listener()}/>
                        </ValidateWrapper>
                    </td>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"环境规格："}</td>
                    <td style="vertical-align: top;">
                        {
                            if id.is_none() {
                                let onchange = {
                                    let environment_schema_detail = environment_schema_detail.clone();
                                    let schema_resource_list = schema_resource_list.clone();
                                    let validator = edit_form.environment_schema_id.listener();
                                    UnsyncCallback::new(move |option: Option<(Id, String)>| {
                                        if let Some((id, _name)) = option {
                                            let environment_schema_detail = environment_schema_detail.clone();
                                            let schema_resource_list = schema_resource_list.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                handle_environment_schema_change(
                                                    id,
                                                    &environment_schema_detail,
                                                    &schema_resource_list,
                                                )
                                                .await.ok();
                                            });
                                            validator.run(Some(id));
                                        } else {
                                            validator.run(None);
                                        }
                                    })
                                };
                                view! {
                                    <ValidateWrapper error={edit_form.environment_schema_id.error()}>
                                        <Selection value={edit_form.environment_schema_id.data()} options={environment_schema_list} onchange={onchange}/>
                                    </ValidateWrapper>
                                }.into_any()
                            } else {
                                (move || {
                                    //一旦指定了环境规格之后，就不让修改
                                    if let Some(environment_schema_detail) = environment_schema_detail.read().as_ref() {
                                        environment_schema_detail.name.clone().into_any()
                                    } else {
                                        view! {}.into_any()
                                    }
                                }).into_any()
                            }
                        }
                    </td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:16em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源规格"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        <For
                            each={
                                let schema_resource_list = edit_form.schema_resource_list.clone();
                                move || { schema_resource_list.get() }
                            }
                            key=|(key, _schema_resource)| { key.clone() }
                            children=move |(_key, schema_resource)| {
                                let environment_edit_state = environment_edit_state.clone();
                                let extension_id = schema_resource.extension_id.clone();
                                let extension_list = extension_list.clone();
                                let schema_resource_id = schema_resource.id;
                                let schema_resource_name = schema_resource.name.clone();
                                let is_active = {
                                    let active_schema_resource_id = edit_form.active_schema_resource_id.clone();
                                    move || {
                                        &active_schema_resource_id.read() == &Some(schema_resource_id.clone())
                                    }
                                };
                                let background_color = {
                                    let is_active = is_active.clone();
                                    move || {
                                        if is_active() {
                                            "background-color: #EEE"
                                        } else {
                                            ""
                                        }
                                    }
                                };
                                view! {
                                    <div>
                                        <div style={move || format!("border-bottom: 1px solid #CCC;padding: 0 0.5em;display: flex;justify-content: space-between;align-items: center;{}", background_color())}>
                                            <div on:click={
                                                let active_schema_resource_id = edit_form.active_schema_resource_id.clone();
                                                move |_| {
                                                    let active_schema_resource_id = active_schema_resource_id.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        active_schema_resource_id.set(Some(schema_resource_id));
                                                        utils::wait(0).await;
                                                        utils::trigger_resize();
                                                    });
                                                }
                                            } style="flex-grow: 1;flex-shrink: 1;padding: 0.5em 0;">
                                                {schema_resource_name}
                                            </div>
                                        </div>
                                        <Visable condition={is_active} style="position:absolute;left:16em;right:0;top:0;bottom:0;overflow: auto;">
                                            {
                                                let extension_list = extension_list.clone();
                                                let active_resource_key = edit_form.active_resource_key.clone();
                                                let environment_edit_state = environment_edit_state.clone();
                                                let active_resource_key = active_resource_key.clone();
                                                let active_resource_key_clone = active_resource_key.clone();
                                                let extension_id_clone = extension_id.clone();
                                                let extension_list = extension_list.clone();
                                                let resource_list = schema_resource.resource_list.clone();
                                                view! {
                                                    <div style="width:20em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                                                        <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源列表"}</div>
                                                        <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                                            <For
                                                                each={
                                                                    let resource_list = resource_list.clone();
                                                                    move || { resource_list.get().into_iter().enumerate() }
                                                                }
                                                                key=|(_index, (resource_key, _resource))| { resource_key.clone() }
                                                                children=move |(index, (resource_key, resource))| {
                                                                    let environment_edit_state = environment_edit_state.clone();
                                                                    let extension_id = extension_id_clone.clone();
                                                                    let active_resource_key = active_resource_key.clone();
                                                                    let error = resource.name.error();
                                                                    let name_validators = resource.name.validators();
                                                                    let resource_clone = resource.clone();
                                                                    let on_remove = {
                                                                        let resource_list = resource_list.clone();
                                                                        UnsyncCallback::new(move |_| {
                                                                            resource_list.write().remove(index);
                                                                        })
                                                                    };
                                                                    let name = resource.name.data();
                                                                    view! {
                                                                        <ValidateWrapper error={resource.name.error()}>
                                                                            {
                                                                                let environment_edit_state = environment_edit_state.clone();
                                                                                let extension_id = extension_id.clone();
                                                                                let active_resource_key = active_resource_key.clone();
                                                                                let resource_key = resource_key.clone();
                                                                                let name_validators = name_validators.clone();
                                                                                let resource = resource_clone.clone();
                                                                                let on_remove = on_remove.clone();
                                                                                let extension_id = extension_id.clone();
                                                                                let resource = resource.clone();
                                                                                let on_remove = on_remove.clone();
                                                                                let is_active = {
                                                                                    let active_resource_key = active_resource_key.clone();
                                                                                    let resource_key = resource_key.clone();
                                                                                    move || {
                                                                                        &active_resource_key.read() == &Some(resource_key.clone())
                                                                                    }
                                                                                };
                                                                                let background_color = {
                                                                                    let is_active = is_active.clone();
                                                                                    move || {
                                                                                        if is_active() {
                                                                                            "background-color: #EEE"
                                                                                        } else {
                                                                                            ""
                                                                                        }
                                                                                    }
                                                                                };
                                                                                view! {
                                                                                    <div>
                                                                                        <div style={move || format!("border-bottom: 1px solid #CCC;padding: 0 0.5em;display: flex;justify-content: space-between;align-items: center;{}", background_color())}>
                                                                                            <div on:click={move |_| {
                                                                                                let active_resource_key = active_resource_key.clone();
                                                                                                let resource_key = resource_key.clone();
                                                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                                                    active_resource_key.set(Some(resource_key.clone()));
                                                                                                    utils::wait(0).await;
                                                                                                    utils::trigger_resize();
                                                                                                });
                                                                                            }} style="flex-grow: 1;flex-shrink: 1;padding: 0.5em 0;">
                                                                                                {
                                                                                                    move || {
                                                                                                        let name = name.get();
                                                                                                        if name.is_empty() {
                                                                                                            SharedString::from("(缺少资源名称)")
                                                                                                        } else {
                                                                                                            name
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                            </div>
                                                                                            <Button onclick={on_remove} style={SharedString::from("margin-left:0.5em;")}>{"移除"}</Button>
                                                                                        </div>
                                                                                        <Visable condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;overflow: auto;">
                                                                                            {environment_edit_state.resource_edit_view(extension_id, &resource, name.clone(), error, name_validators)}
                                                                                        </Visable>
                                                                                    </div>
                                                                                }
                                                                            }
                                                                        </ValidateWrapper>
                                                                    }
                                                                }
                                                            />
                                                            <div style="margin-top: 0.5em;">
                                                                <Button onclick={UnsyncCallback::new(move |_| {
                                                                    let configuration_schema = get_configuration_schema(&extension_list.read(), &extension_id)
                                                                    .map(|configuration_schema| configuration_schema.clone())
                                                                    .unwrap_or_default();
                                                                    let new_environment = EnvironmentResource {
                                                                        id: Default::default(),
                                                                        name: init_resource_name(Default::default()),
                                                                        extension_configuration: get_default_config(configuration_schema),
                                                                        test_error: Default::default(),
                                                                    };
                                                                    let new_key: Key = gen_id().into();
                                                                    active_resource_key_clone.set(Some(new_key.clone()));
                                                                    resource_list.write().push((new_key, new_environment));
                                                                })}>{"添加"}</Button>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        </Visable>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </div>
            <div style="margin-top: 0.5em;">
                <Button disabled={is_saving} onclick={on_save}>{"保存"}</Button>
                <Show
                    when={ let err_msg = err_msg.clone(); move || { err_msg.read().is_some() } }
                >
                    <span class="middle" style="color:red;margin-left: 0.5em;">{err_msg}</span>
                </Show>
            </div>
        </div>
    }
}

impl EnvironmentEditState {
    fn resource_edit_view(
        &self,
        extension_id: String,
        resource: &EnvironmentResource,
        name: RwSignal<SharedString>,
        error: RwSignal<Option<SharedString>>,
        name_validators: Arc<Validators<SharedString>>,
    ) -> impl IntoView + use<> {
        let extension_configuration = resource.extension_configuration.clone();
        view! {
            <div style="padding: 0.25em;">
                <table>
                    <tr>
                        <td class="align-right" style="vertical-align: top;"><Required/>{"资源名称"}</td>
                        <td>
                            <ValidateWrapper error={error.clone()} style="display:inline-block;">
                                <Input value={name} onupdate={UnsyncCallback::new(move |value| {
                                    name_validators.validate_into(&value, &error)
                                })}/>
                            </ValidateWrapper>
                        </td>
                    </tr>
                    <tr>
                        <td class="align-right" style="vertical-align: top;">{"资源配置"}</td>
                        <td>
                            <ConfigView attributes={extension_configuration.clone()}/>
                        </td>
                    </tr>
                    <tr>
                        <td class="align-right" style="vertical-align: top;">{"测试配置"}</td>
                        <td>
                            {
                                let on_test = {
                                    let test_error = resource.test_error.clone();
                                    UnsyncCallback::new(move |_| {
                                        test_error.set(None);
                                        test_configuration(extension_id.clone(), extension_configuration.clone(), test_error.clone());
                                    })
                                };
                                view! {
                                    <div>
                                        <Button onclick={on_test}>{"测试"}</Button>
                                        {
                                            let test_error = resource.test_error.clone();
                                            move || {
                                                if let Some(test_error) = test_error.get() {
                                                    match test_error {
                                                        Ok(_) => {
                                                            view! {
                                                                <span style="color: green;">{"测试成功!"}</span>
                                                            }.into_any()
                                                        },
                                                        Err(err) => {
                                                            view! {
                                                                <span style="color: red;">{err}</span>
                                                            }.into_any()
                                                        }
                                                    }
                                                } else {
                                                    view! {}.into_any()
                                                }
                                            }
                                        }
                                    </div>
                                }
                            }
                        </td>
                    </tr>
                </table>
            </div>
        }
    }
}

async fn query_environment_schema_list(
    list: &RwSignal<Vec<EnvironmentSchema>>,
) -> Result<Vec<EnvironmentSchema>, SharedString> {
    let pagination_list = QueryEnvironmentSchemaApi
        .call(&QueryEnvironmentSchemaReq {
            page_no: Some(1),
            ..QueryEnvironmentSchemaReq::empty()
        })
        .await?;
    list.set(pagination_list.list.clone());
    return Ok(pagination_list.list);
}

async fn query_extension_list(
    extension_list: &RwSignal<Vec<Extension>>,
) -> Result<Vec<Extension>, SharedString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
}

async fn read_environment_schema_detail(
    detail: &RwSignal<Option<EnvironmentSchemaDetail>>,
    environment_schema_id: Id,
) -> Result<EnvironmentSchemaDetail, SharedString> {
    let params = ReadEnvironmentSchemaReq {
        id: environment_schema_id,
    };
    let environment_schema = ReadEnvironmentSchemaApi.call(&params).await?;
    detail.set(Some(environment_schema.clone()));
    return Ok(environment_schema);
}

fn init_resource_name(value: SharedString) -> ValidateData<SharedString> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请输入资源名称"))),
    )
}

async fn handle_environment_schema_change(
    environment_schema_id: Id,
    detail: &RwSignal<Option<EnvironmentSchemaDetail>>,
    schema_resource_list: &RwSignal<Vec<(Key, EnvironmentSchemaResource)>>,
) -> Result<(), SharedString> {
    let environment_schema_detail =
        read_environment_schema_detail(detail, environment_schema_id).await?;
    schema_resource_list.set(
        environment_schema_detail
            .resource_list
            .iter()
            .map(|schema_resource| {
                (
                    utils::gen_id().into(),
                    EnvironmentSchemaResource {
                        id: schema_resource.id,
                        extension_id: schema_resource.extension_id.clone(),
                        name: schema_resource.name.clone(),
                        resource_list: Default::default(),
                    },
                )
            })
            .collect(),
    );
    return Ok(());
}

async fn read_environment_detail(
    edit_form: &EditForm,
    extension_list: &[Extension],
    id: Id,
) -> Result<Environment, SharedString> {
    let params = ReadEnvironmentReq { id: id };
    let environment = ReadEnvironmentApi.call(&params).await?;
    edit_form
        .environment_schema_id
        .set(environment.environment_schema_id.into());
    edit_form.name.set(environment.name.clone().into());
    edit_form.schema_resource_list.set(
        environment
            .schema_resource_list
            .iter()
            .map(|schema_resource| {
                let configuration_schema =
                    get_configuration_schema(extension_list, &schema_resource.extension_id)
                        .map(|configuration_schema| configuration_schema.clone())
                        .unwrap_or_default();
                (
                    utils::gen_id().into(),
                    EnvironmentSchemaResource {
                        id: schema_resource.id,
                        name: schema_resource.name.clone(),
                        extension_id: schema_resource.extension_id.clone(),
                        resource_list: RwSignal::new(
                            schema_resource
                                .resource_list
                                .iter()
                                .map(move |resource| {
                                    let extension_configuration = parse_config(
                                        configuration_schema.clone(),
                                        &resource.extension_configuration,
                                    );
                                    (
                                        utils::gen_id().into(),
                                        EnvironmentResource {
                                            id: Some(resource.id),
                                            name: init_resource_name(resource.name.clone().into()),
                                            extension_configuration: extension_configuration, //扩展配置
                                            test_error: Default::default(),
                                        },
                                    )
                                })
                                .collect(),
                        ),
                    },
                )
            })
            .collect(),
    );
    return Ok(environment);
}

fn test_configuration(
    extension_id: String,
    extension_configuration: Vec<(Key, Attribute, AttributeValue)>,
    test_error: RwSignal<Option<Result<(), SharedString>>>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        try_test_configuration(extension_id, &extension_configuration, &test_error)
            .await
            .ok();
    });
}

async fn try_test_configuration(
    extension_id: String,
    extension_configuration: &[(Key, Attribute, AttributeValue)],
    test_error: &RwSignal<Option<Result<(), SharedString>>>,
) -> Result<(), SharedString> {
    let err_msgs = chk_single_err(extension_configuration).await;
    if let Some(first) = err_msgs.first() {
        return Err(first.clone());
    }
    upload_single_files(extension_configuration).await?;
    let extension_configuration = serialize_config(extension_configuration);
    let ret = TestConfigurationApi
        .call(&TestConfigurationReq {
            extension_id: extension_id,
            extension_configuration: extension_configuration,
        })
        .await;
    test_error.set(Some(ret));
    return Ok(());
}

async fn chk_single_err(
    extension_configuration: &[(Key, Attribute, AttributeValue)],
) -> Vec<SharedString> {
    let mut err_msgs: Vec<SharedString> = Vec::new();
    for (_, _, value) in extension_configuration.iter() {
        if let Err(error) = value.validate(true) {
            err_msgs.push(error);
        }
    }
    return err_msgs;
}

async fn upload_single_files(
    extension_configuration: &[(Key, Attribute, AttributeValue)],
) -> Result<(), SharedString> {
    let mut files = Vec::new();
    for (_, _, attr_value) in extension_configuration {
        match attr_value {
            // AttributeValue::RichText(value) => {
            //     let value = value.get();
            //     upload_resource(&value).await.map_err(|err| {
            //         log::error!("上传富文本资源失败: {:?}", err);
            //         return SharedString::from("上传文件失败");
            //     })?;
            // }
            AttributeValue::File(file_value) => {
                if let Some(file) = file_value.get() {
                    match file {
                        Resource::Local(hashing_file) => {
                            let handle = file_value.clone();
                            files.push((
                                hashing_file.clone(),
                                UnsyncCallback::new(move |result| match result {
                                    Ok(metadata) => {
                                        handle.set(Some(Resource::Remote(metadata)));
                                    }
                                    Err(err) => {
                                        log::error!("上传文件失败: {:?}", err);
                                    }
                                }),
                            ));
                        }
                        Resource::Remote(_metadata) => (),
                    }
                }
            }
            AttributeValue::FileList(file_list) => {
                let new_files = file_list.get();
                let file_count = new_files.len();
                let finished_count = 0;
                let lock_data = Arc::new(Mutex::new((new_files, finished_count)));
                for (index, (_key, file, ())) in file_list.get().into_iter().enumerate() {
                    let lock_data = lock_data.clone();
                    match file.get() {
                        Resource::Local(hashing_file) => {
                            let file_list = file_list.clone();
                            files.push((
                                hashing_file.clone(),
                                UnsyncCallback::new(move |result| {
                                    let mut lock_data = lock_data.lock().unwrap();
                                    let finished_count = lock_data.1 + 1;
                                    lock_data.1 = finished_count;
                                    let new_files = &mut lock_data.0;
                                    match result {
                                        Ok(metadata) => {
                                            new_files[index].1.set(Resource::Remote(metadata));
                                        }
                                        Err(err) => {
                                            log::error!("上传文件失败: {:?}", err);
                                        }
                                    }
                                    if finished_count == file_count {
                                        file_list.set(new_files.clone());
                                    }
                                }),
                            ));
                        }
                        Resource::Remote(_metadata) => (),
                    }
                }
            }
            _ => (),
        }
    }
    upload_files(files).await?;
    //延迟0秒，让修改的值生效
    utils::wait(0).await;
    return Ok(());
}

async fn chk_form_err(id: Option<Id>, edit_form: &EditForm) -> Vec<SharedString> {
    let mut err_msgs: Vec<SharedString> = Vec::new();
    if id.is_none() {
        if let Err(error) = edit_form.environment_schema_id.validate(true) {
            err_msgs.push(error);
        }
    }
    if let Err(error) = edit_form.name.validate(true) {
        err_msgs.push(error);
    }
    let mut empty_id = None;
    let mut active_ids = None;
    for (_key, schema_resource) in edit_form.schema_resource_list.get().iter() {
        let schema_resource_list = schema_resource.resource_list.get();
        if schema_resource_list.is_empty() {
            err_msgs.push(SharedString::from(format!(
                "没有添加\"{}\"对应的资源",
                schema_resource.name
            )));
            if empty_id.is_none() {
                empty_id.replace(schema_resource.id);
            }
        } else {
            for (resource_key, resource) in schema_resource_list.iter() {
                if let Err(error) = resource.name.validate(true) {
                    err_msgs.push(error);
                    if active_ids.is_none() {
                        active_ids.replace((schema_resource.id, resource_key.clone()));
                    }
                }
                for (_, _, value) in resource.extension_configuration.iter() {
                    if let Err(error) = value.validate(true) {
                        err_msgs.push(error);
                        if active_ids.is_none() {
                            active_ids.replace((schema_resource.id, resource_key.clone()));
                        }
                    }
                }
            }
        }
    }
    if let Some((schema_resource_id, resource_key)) = active_ids {
        edit_form
            .active_schema_resource_id
            .set(Some(schema_resource_id));
        edit_form.active_resource_key.set(Some(resource_key));
        utils::wait(0).await;
        utils::trigger_resize();
    } else if let Some(schema_resource_id) = empty_id {
        edit_form
            .active_schema_resource_id
            .set(Some(schema_resource_id));
        edit_form.active_resource_key.set(None);
    }
    return err_msgs;
}

async fn try_upload_files(edit_form: &EditForm) -> Result<(), SharedString> {
    let mut files = Vec::new();
    for (_key, schema_resource) in edit_form.schema_resource_list.get().iter() {
        let resource_list = schema_resource.resource_list.get();
        for (_, resource) in resource_list.iter() {
            for (_, _, attr_value) in &resource.extension_configuration {
                match attr_value {
                    // AttributeValue::RichText(value) => {
                    //     let value = value.get();
                    //     upload_resource(&value).await.map_err(|err| {
                    //         log::error!("上传富文本资源失败: {:?}", err);
                    //         return SharedString::from("上传文件失败");
                    //     })?;
                    // }
                    AttributeValue::File(file_value) => {
                        if let Some(file) = file_value.get() {
                            match file {
                                Resource::Local(hashing_file) => {
                                    let handle = file_value.clone();
                                    files.push((
                                        hashing_file.clone(),
                                        UnsyncCallback::new(move |result| match result {
                                            Ok(metadata) => {
                                                handle.set(Some(Resource::Remote(metadata)));
                                            }
                                            Err(err) => {
                                                log::error!("上传文件失败: {:?}", err);
                                            }
                                        }),
                                    ));
                                }
                                Resource::Remote(_metadata) => (),
                            }
                        }
                    }
                    AttributeValue::FileList(file_list) => {
                        let new_files = file_list.get();
                        let file_count = new_files.len();
                        let finished_count = 0;
                        let lock_data = Arc::new(Mutex::new((new_files, finished_count)));
                        for (index, (_key, file, ())) in file_list.get().into_iter().enumerate() {
                            let lock_data = lock_data.clone();
                            match file.get() {
                                Resource::Local(hashing_file) => {
                                    let file_list = file_list.clone();
                                    files.push((
                                        hashing_file.clone(),
                                        UnsyncCallback::new(move |result| {
                                            let mut lock_data = lock_data.lock().unwrap();
                                            let finished_count = lock_data.1 + 1;
                                            lock_data.1 = finished_count;
                                            let new_files = &mut lock_data.0;
                                            match result {
                                                Ok(metadata) => {
                                                    new_files[index]
                                                        .1
                                                        .set(Resource::Remote(metadata));
                                                }
                                                Err(err) => {
                                                    log::error!("上传文件失败: {:?}", err);
                                                }
                                            }
                                            if finished_count == file_count {
                                                file_list.set(new_files.clone());
                                            }
                                        }),
                                    ));
                                }
                                Resource::Remote(_metadata) => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    upload_files(files).await?;
    //延迟0秒，让修改的值生效
    utils::wait(0).await;
    return Ok(());
}

fn collect_resource_list(
    edit_form: &EditForm,
) -> Vec<(Id, String, Vec<(Option<Id>, String, String)>)> {
    let list = edit_form.schema_resource_list.get();
    let mut schema_resource_list: Vec<_> = Vec::with_capacity(list.len());
    for (_key, schema_resource) in list.iter() {
        schema_resource_list.push((
            schema_resource.id,
            schema_resource.extension_id.clone(),
            schema_resource
                .resource_list
                .get()
                .iter()
                .map(|(_, resource)| {
                    let extension_configuration =
                        serialize_config(&resource.extension_configuration);
                    (
                        resource.id,
                        resource.name.get().to_string(),
                        extension_configuration,
                    )
                })
                .collect(),
        ));
    }
    return schema_resource_list;
}

async fn save_environment(
    id: Option<Id>,
    edit_form: &EditForm,
    is_saving: RwSignal<bool>,
    err_msg: &RwSignal<Option<SharedString>>,
    onsave: &Option<UnsyncCallback<PrimaryKey>>,
) -> Result<(), SharedString> {
    let err_msgs = chk_form_err(id, edit_form).await;
    if let Some(first) = err_msgs.first() {
        err_msg.set(Some(first.clone()));
        return Err(first.clone());
    }
    if let Err(err) = try_upload_files(edit_form).await {
        err_msg.set(Some(err.clone()));
        return Err(err);
    }
    let schema_resource_list = collect_resource_list(edit_form);
    if let Some(id) = id {
        let ret = UpdateEnvironmentApi
            .lock_handler(is_saving)
            .call(&UpdateEnvironmentReq {
                id: id,
                name: edit_form.name.get().to_string(),
                schema_resource_list: schema_resource_list
                    .into_iter()
                    .map(|(id, extension_id, resource_list)| {
                        sdk::environment::update_environment::EnvironmentSchemaResource {
                            id: id,
                            extension_id: extension_id,
                            resource_list: resource_list
                                .into_iter()
                                .map(|(id, name, extension_configuration)| {
                                    sdk::environment::update_environment::EnvironmentResource {
                                        id: id,
                                        name: name,
                                        extension_configuration: extension_configuration, //扩展配置
                                    }
                                })
                                .collect(),
                        }
                    })
                    .collect(),
            })
            .await;
        match ret {
            Err(err) => {
                log::error!("{}", err);
                err_msg.set(Some(err));
            }
            Ok(_) => {
                match onsave {
                    Some(onsave) => {
                        onsave.run(tihu::PrimaryKey { id: id });
                    }
                    None => (),
                }
                utils::success(SharedString::from("保存成功"));
            }
        }
    } else {
        let ret = InsertEnvironmentApi
            .lock_handler(is_saving)
            .call(&InsertEnvironmentReq {
                environment_schema_id: edit_form.environment_schema_id.get().unwrap(),
                name: edit_form.name.get().to_string(),
                schema_resource_list: schema_resource_list
                    .into_iter()
                    .map(|(id, extension_id, resource_list)| {
                        sdk::environment::insert_environment::EnvironmentSchemaResource {
                            id: id,
                            extension_id: extension_id,
                            resource_list: resource_list
                                .into_iter()
                                .map(|(_id, name, extension_configuration)| {
                                    sdk::environment::insert_environment::EnvironmentResource {
                                        name: name,
                                        extension_configuration: extension_configuration, //扩展配置
                                    }
                                })
                                .collect(),
                        }
                    })
                    .collect(),
            })
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
    }
    return Ok(());
}
