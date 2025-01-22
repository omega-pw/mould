use super::super::extension::config_view;
use super::super::extension::get_configuration_schema;
use super::super::extension::get_default_config;
use super::super::extension::parse_config;
use super::super::extension::serialize_config;
use super::super::extension::AttributeValue;
use crate::components::button::Button;
use crate::components::input::BindingInput;
use crate::components::required::Required;
use crate::components::rich_text::upload_resource;
use crate::components::selection::BindingSelection;
use crate::components::show::Show;
use crate::components::uploading_files::upload_files;
use crate::components::validate_wrapper::ValidateData;
use crate::components::validate_wrapper::ValidateWrapper;
use crate::components::File;
use crate::sdk;
use crate::utils;
use crate::utils::binding::Binding;
use crate::utils::gen_id;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::LightString;
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
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use tihu::Id;
use tihu::PrimaryKey;
use yew::prelude::*;
use yew::virtual_dom::Key;

type EnvironmentSchemaSelection = BindingSelection<(Id, String)>;

#[derive(Clone, PartialEq, Debug)]
pub struct EnvironmentResource {
    id: Option<Id>,
    name: ValidateData<LightString>,
    extension_configuration: Vec<(Key, Attribute, AttributeValue)>, //扩展配置
    test_error: Binding<Option<Result<(), LightString>>>,
}

/**
 * 环境规格资源
 */
#[derive(Clone, PartialEq, Debug)]
pub struct EnvironmentSchemaResource {
    id: Id,
    extension_id: String,
    name: String,
    resource_list: Binding<Vec<(Key, EnvironmentResource)>>,
}

#[derive(Clone)]
struct EditForm {
    active_schema_resource_id: UseStateHandle<Option<Id>>,
    active_resource_key: UseStateHandle<Option<Key>>,
    environment_schema_id: ValidateData<Option<Id>>,
    name: ValidateData<LightString>,
    schema_resource_list: UseStateHandle<Vec<(Key, EnvironmentSchemaResource)>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Option<Id>,
    #[prop_or_default]
    pub onsave: Option<Callback<PrimaryKey>>,
}

#[derive(Clone)]
struct EnvironmentEditState {
    is_saving: UseStateHandle<bool>,
    err_msg: UseStateHandle<Option<LightString>>,
    environment_schema_list: UseStateHandle<Vec<EnvironmentSchema>>,
    environment_schema_detail: UseStateHandle<Option<EnvironmentSchemaDetail>>,
    extension_list: UseStateHandle<Vec<Extension>>,
    edit_form: EditForm,
}

#[function_component]
pub fn EnvironmentEdit(props: &Props) -> Html {
    let is_saving: UseStateHandle<bool> = use_state(|| false);
    let err_msg: UseStateHandle<Option<LightString>> = use_state(|| None);
    let environment_schema_list: UseStateHandle<Vec<EnvironmentSchema>> = use_state(|| Vec::new());
    let environment_schema_detail: UseStateHandle<Option<EnvironmentSchemaDetail>> =
        use_state(|| None);
    let extension_list: UseStateHandle<Vec<Extension>> = use_state(|| Default::default());
    let edit_form = EditForm {
        active_schema_resource_id: use_state(|| Default::default()),
        active_resource_key: use_state(|| Default::default()),
        environment_schema_id: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请选择环境规格"))),
        ),
        name: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请输入环境名称"))),
        ),
        schema_resource_list: use_state(|| Vec::new()),
    };
    let environment_edit_state = EnvironmentEditState {
        is_saving: is_saving.clone(),
        err_msg: err_msg.clone(),
        environment_schema_list: environment_schema_list.clone(),
        environment_schema_detail: environment_schema_detail.clone(),
        extension_list: extension_list.clone(),
        edit_form: edit_form.clone(),
    };
    let id = props.id;
    let environment_schema_id = edit_form.environment_schema_id.clone();
    let schema_resource_list = edit_form.schema_resource_list.clone();
    let schema_resource_list_clone = edit_form.schema_resource_list.clone();
    let edit_form_clone = edit_form.clone();
    let environment_schema_list_clone = environment_schema_list.clone();
    let extension_list_clone = extension_list.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let environment_schema_detail_clone2 = environment_schema_detail.clone();
    use_effect_with(id, move |_| {
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
                            utils::error(LightString::from("请先添加环境规格"));
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
        || ()
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
            save_environment(id, &edit_form, is_saving, &err_msg, &onsave)
                .await
                .ok();
        });
    });
    let environment_schema_list: Vec<_> = environment_schema_list
        .iter()
        .map(|item| (item.id.clone().into(), item.name.clone()))
        .collect();
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"环境名称："}</td>
                    <td>
                        {
                            edit_form.name.view(move |name: UseStateHandle<LightString>, validator| {
                                html! {
                                    <BindingInput value={name} onupdate={validator}/>
                                }
                            })
                        }
                    </td>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"环境规格："}</td>
                    <td style="vertical-align: top;">
                        {
                            if id.is_none() {
                                let environment_schema_detail = environment_schema_detail.clone();
                                let schema_resource_list = schema_resource_list.clone();
                                {
                                    edit_form.environment_schema_id.view(move |environment_schema_id, validator: Callback<Option<Id>>| {
                                        let environment_schema_detail = environment_schema_detail.clone();
                                        let schema_resource_list = schema_resource_list.clone();
                                        let onchange = Callback::from(move |option: Option<(Id, String)>| {
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
                                                validator.emit(Some(id));
                                            } else {
                                                validator.emit(None);
                                            }
                                        });
                                        html! {
                                            <EnvironmentSchemaSelection value={environment_schema_id} options={environment_schema_list.clone()} onchange={onchange}/>
                                        }
                                    })
                                }
                            } else {
                                //一旦指定了环境规格之后，就不让修改
                                if let Some(environment_schema_detail) = environment_schema_detail.as_ref() {
                                    html! {environment_schema_detail.name.clone()}
                                } else {
                                    html!{}
                                }
                            }
                        }
                    </td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                <div style="width:16em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                    <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源规格"}</div>
                    <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                        {
                            for edit_form.schema_resource_list.iter().map(|(key, schema_resource)| {
                                let environment_edit_state = environment_edit_state.clone();
                                let extension_id = schema_resource.extension_id.clone();
                                let extension_list = extension_list.clone();
                                let schema_resource_id = schema_resource.id;
                                let schema_resource_name = schema_resource.name.clone();
                                let active_schema_resource_id = edit_form.active_schema_resource_id.clone();
                                let is_active = active_schema_resource_id.deref() == &Some(schema_resource_id);
                                let background_color = if is_active {
                                    "background-color: #EEE"
                                } else {
                                    ""
                                };
                                html! {
                                    <div key={key.clone()}>
                                        <div style={format!("border-bottom: 1px solid #CCC;padding: 0 0.5em;display: flex;justify-content: space-between;align-items: center;{}", background_color)}>
                                            <div onclick={Callback::from(move |_| {
                                                let active_schema_resource_id = active_schema_resource_id.clone();
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    active_schema_resource_id.set(Some(schema_resource_id));
                                                    utils::wait(0).await;
                                                    utils::trigger_resize();
                                                });
                                            })} style="flex-grow: 1;flex-shrink: 1;padding: 0.5em 0;">
                                                {schema_resource_name}
                                            </div>
                                        </div>
                                        <Show condition={is_active} style="position:absolute;left:16em;right:0;top:0;bottom:0;overflow: auto;">
                                            {
                                                {
                                                    let extension_list = extension_list.clone();
                                                    let active_resource_key = edit_form.active_resource_key.clone();
                                                    schema_resource.resource_list.view(move |resource_list: UseStateHandle<Vec<(Key, EnvironmentResource)>>| {
                                                        let environment_edit_state = environment_edit_state.clone();
                                                        let active_resource_key = active_resource_key.clone();
                                                        let active_resource_key_clone = active_resource_key.clone();
                                                        let extension_id = extension_id.clone();
                                                        let extension_list = extension_list.clone();
                                                        let resource_list_clone = resource_list.clone();
                                                        html! {
                                                            <div style="width:20em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                                                                <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"资源列表"}</div>
                                                                <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                                                    {
                                                                        for resource_list.deref().clone().into_iter().enumerate().map(|(index, (resource_key, resource))| {
                                                                            let environment_edit_state = environment_edit_state.clone();
                                                                            let extension_id = extension_id.clone();
                                                                            let active_resource_key = active_resource_key.clone();
                                                                            let error = resource.name.error.clone();
                                                                            let name_validators = resource.name.validators.clone();
                                                                            let resource_list = resource_list_clone.clone();
                                                                            let resource_clone = resource.clone();
                                                                            let on_remove = Callback::from(move |_| {
                                                                                let mut new_items = resource_list.deref().clone();
                                                                                new_items.remove(index);
                                                                                resource_list.set(new_items);
                                                                            });
                                                                            html! {
                                                                                <div key={resource_key.clone()}>
                                                                                    {
                                                                                        resource.name.data.view(move |name: UseStateHandle<AttrValue>| {
                                                                                            let environment_edit_state = environment_edit_state.clone();
                                                                                            let extension_id = extension_id.clone();
                                                                                            let active_resource_key = active_resource_key.clone();
                                                                                            let resource_key = resource_key.clone();
                                                                                            let name_validators = name_validators.clone();
                                                                                            let resource = resource_clone.clone();
                                                                                            let on_remove = on_remove.clone();
                                                                                            error.view(move |error: UseStateHandle<Option<AttrValue>>| {
                                                                                                let extension_id = extension_id.clone();
                                                                                                let active_resource_key = active_resource_key.clone();
                                                                                                let resource_key = resource_key.clone();
                                                                                                let name_validators = name_validators.clone();
                                                                                                let resource = resource.clone();
                                                                                                let on_remove = on_remove.clone();
                                                                                                let is_active = active_resource_key.deref() == &Some(resource_key.clone());
                                                                                                let background_color = if is_active {
                                                                                                    "background-color: #EEE"
                                                                                                } else {
                                                                                                    ""
                                                                                                };
                                                                                                html! {
                                                                                                    <div>
                                                                                                        <div style={format!("border-bottom: 1px solid #CCC;padding: 0 0.5em;display: flex;justify-content: space-between;align-items: center;{}", background_color)}>
                                                                                                            <div onclick={Callback::from(move |_| {
                                                                                                                let active_resource_key = active_resource_key.clone();
                                                                                                                let resource_key = resource_key.clone();
                                                                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                                                                    active_resource_key.set(Some(resource_key.clone()));
                                                                                                                    utils::wait(0).await;
                                                                                                                    utils::trigger_resize();
                                                                                                                });
                                                                                                            })} style="flex-grow: 1;flex-shrink: 1;padding: 0.5em 0;">
                                                                                                                {
                                                                                                                    if name.is_empty() {
                                                                                                                        LightString::from("(缺少资源名称)")
                                                                                                                    } else {
                                                                                                                        name.deref().clone()
                                                                                                                    }
                                                                                                                }
                                                                                                            </div>
                                                                                                            <Button onclick={on_remove} style="margin-left:0.5em;">{"移除"}</Button>
                                                                                                        </div>
                                                                                                        <Show condition={is_active} style="position:absolute;left:20em;right:0;top:0;bottom:0;overflow: auto;">
                                                                                                            {environment_edit_state.resource_edit_view(extension_id, &resource, name.clone(), error, name_validators)}
                                                                                                        </Show>
                                                                                                    </div>
                                                                                                }
                                                                                            })
                                                                                        })
                                                                                    }
                                                                                </div>
                                                                            }
                                                                        })
                                                                    }
                                                                    <div style="margin-top: 0.5em;">
                                                                        <Button onclick={Callback::from(move |_| {
                                                                            let configuration_schema = get_configuration_schema(&extension_list, &extension_id)
                                                                            .map(|configuration_schema| configuration_schema.clone())
                                                                            .unwrap_or_default();
                                                                            let new_environment = EnvironmentResource {
                                                                                id: Default::default(),
                                                                                name: init_resource_name(Default::default()),
                                                                                extension_configuration: get_default_config(configuration_schema),
                                                                                test_error: Default::default(),
                                                                            };
                                                                            let mut new_list = resource_list.deref().clone();
                                                                            let new_key: Key = gen_id().into();
                                                                            active_resource_key_clone.set(Some(new_key.clone()));
                                                                            new_list.push((new_key, new_environment));
                                                                            resource_list.set(new_list);
                                                                        })}>{"添加"}</Button>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                }
                                            }
                                        </Show>
                                    </div>
                                }
                            })
                        }
                    </div>
                </div>
            </div>
            <div style="margin-top: 0.5em;">
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
            </div>
        </div>
    }
}

impl EnvironmentEditState {
    fn resource_edit_view(
        &self,
        extension_id: String,
        resource: &EnvironmentResource,
        name: UseStateHandle<LightString>,
        error: UseStateHandle<Option<LightString>>,
        name_validators: Validators<LightString>,
    ) -> Html {
        let extension_configuration = resource.extension_configuration.clone();
        html! {
            <div style="padding: 0.25em;">
                <table>
                    <tr>
                        <td class="align-right" style="vertical-align: top;"><Required/>{"资源名称"}</td>
                        <td>
                            <ValidateWrapper error={error.deref().clone()} style="display:inline-block;">
                                <BindingInput value={name} onupdate={Callback::from(move |value| {
                                    name_validators.validate_into(&value, &error)
                                })}/>
                            </ValidateWrapper>
                        </td>
                    </tr>
                    <tr>
                        <td class="align-right" style="vertical-align: top;">{"资源配置"}</td>
                        <td>
                            { config_view(&resource.extension_configuration) }
                        </td>
                    </tr>
                    <tr>
                        <td class="align-right" style="vertical-align: top;">{"测试配置"}</td>
                        <td>
                            {
                                resource.test_error.view(move |test_error: UseStateHandle<Option<Result<(), LightString>>>| {
                                    let extension_id = extension_id.clone();
                                    let extension_configuration = extension_configuration.clone();
                                    let on_test = {
                                        let test_error = test_error.clone();
                                        Callback::from(move |_| {
                                            test_error.set(None);
                                            test_configuration(extension_id.clone(), extension_configuration.clone(), test_error.clone());
                                        })
                                    };
                                    html! {
                                        <div>
                                            <Button onclick={on_test}>{"测试"}</Button>
                                            {
                                                if let Some(test_error) = test_error.deref() {
                                                    match test_error {
                                                        Ok(_) => {
                                                            html! {
                                                                <span style="color: green;">{"测试成功!"}</span>
                                                            }
                                                        },
                                                        Err(err) => {
                                                            html! {
                                                                <span style="color: red;">{err}</span>
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                    }
                                })
                            }
                        </td>
                    </tr>
                </table>
            </div>
        }
    }
}

async fn query_environment_schema_list(
    list: &UseStateHandle<Vec<EnvironmentSchema>>,
) -> Result<Vec<EnvironmentSchema>, LightString> {
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
    extension_list: &UseStateHandle<Vec<Extension>>,
) -> Result<Vec<Extension>, LightString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
}

async fn read_environment_schema_detail(
    detail: &UseStateHandle<Option<EnvironmentSchemaDetail>>,
    environment_schema_id: Id,
) -> Result<EnvironmentSchemaDetail, LightString> {
    let params = ReadEnvironmentSchemaReq {
        id: environment_schema_id,
    };
    let environment_schema = ReadEnvironmentSchemaApi.call(&params).await?;
    detail.set(Some(environment_schema.clone()));
    return Ok(environment_schema);
}

fn init_resource_name(value: LightString) -> ValidateData<LightString> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请输入资源名称"))),
    )
}

async fn handle_environment_schema_change(
    environment_schema_id: Id,
    detail: &UseStateHandle<Option<EnvironmentSchemaDetail>>,
    schema_resource_list: &UseStateHandle<Vec<(Key, EnvironmentSchemaResource)>>,
) -> Result<(), LightString> {
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
) -> Result<Environment, LightString> {
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
                        resource_list: Binding::new(
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
    test_error: UseStateHandle<Option<Result<(), LightString>>>,
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
    test_error: &UseStateHandle<Option<Result<(), LightString>>>,
) -> Result<(), LightString> {
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
) -> Vec<LightString> {
    let mut err_msgs: Vec<LightString> = Vec::new();
    for (_, _, value) in extension_configuration.iter() {
        if let Err(error) = value.validate(true) {
            err_msgs.push(error);
        }
    }
    return err_msgs;
}

async fn upload_single_files(
    extension_configuration: &[(Key, Attribute, AttributeValue)],
) -> Result<(), LightString> {
    let mut files = Vec::new();
    for (_, _, attr_value) in extension_configuration {
        match attr_value {
            // AttributeValue::RichText(value) => {
            //     let value = value.get();
            //     upload_resource(&value).await.map_err(|err| {
            //         log::error!("上传富文本资源失败: {:?}", err);
            //         return LightString::from("上传文件失败");
            //     })?;
            // }
            AttributeValue::File(file_value) => {
                if let Some(file) = file_value.get() {
                    match file {
                        File::Local(hashing_file) => {
                            let handle = file_value.clone();
                            files.push((
                                hashing_file.clone(),
                                Callback::from(move |result| match result {
                                    Ok(key) => {
                                        handle.set(Some(File::Remote {
                                            key: key,
                                            name: hashing_file.file.name(),
                                            size: hashing_file.file.size(),
                                            mime_type: hashing_file.file.type_(),
                                        }));
                                    }
                                    Err(err) => {
                                        log::error!("上传文件失败: {:?}", err);
                                    }
                                }),
                            ));
                        }
                        File::Remote { .. } => (),
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
                    match file {
                        File::Local(hashing_file) => {
                            let file_list = file_list.clone();
                            files.push((
                                hashing_file.clone(),
                                Callback::from(move |result| {
                                    let mut lock_data = lock_data.lock().unwrap();
                                    let finished_count = lock_data.1 + 1;
                                    lock_data.1 = finished_count;
                                    let new_files = &mut lock_data.0;
                                    match result {
                                        Ok(key) => {
                                            new_files[index].1 = File::Remote {
                                                key: key,
                                                name: hashing_file.file.name(),
                                                size: hashing_file.file.size(),
                                                mime_type: hashing_file.file.type_(),
                                            };
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
                        File::Remote { .. } => (),
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

async fn chk_form_err(id: Option<Id>, edit_form: &EditForm) -> Vec<LightString> {
    let mut err_msgs: Vec<LightString> = Vec::new();
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
    for (_key, schema_resource) in edit_form.schema_resource_list.iter() {
        let schema_resource_list = schema_resource.resource_list.get();
        if schema_resource_list.is_empty() {
            err_msgs.push(LightString::from(format!(
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

async fn try_upload_files(edit_form: &EditForm) -> Result<(), LightString> {
    let mut files = Vec::new();
    for (_key, schema_resource) in edit_form.schema_resource_list.iter() {
        let resource_list = schema_resource.resource_list.get();
        for (_, resource) in resource_list.iter() {
            for (_, _, attr_value) in &resource.extension_configuration {
                match attr_value {
                    // AttributeValue::RichText(value) => {
                    //     let value = value.get();
                    //     upload_resource(&value).await.map_err(|err| {
                    //         log::error!("上传富文本资源失败: {:?}", err);
                    //         return LightString::from("上传文件失败");
                    //     })?;
                    // }
                    AttributeValue::File(file_value) => {
                        if let Some(file) = file_value.get() {
                            match file {
                                File::Local(hashing_file) => {
                                    let handle = file_value.clone();
                                    files.push((
                                        hashing_file.clone(),
                                        Callback::from(move |result| match result {
                                            Ok(key) => {
                                                handle.set(Some(File::Remote {
                                                    key: key,
                                                    name: hashing_file.file.name(),
                                                    size: hashing_file.file.size(),
                                                    mime_type: hashing_file.file.type_(),
                                                }));
                                            }
                                            Err(err) => {
                                                log::error!("上传文件失败: {:?}", err);
                                            }
                                        }),
                                    ));
                                }
                                File::Remote { .. } => (),
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
                            match file {
                                File::Local(hashing_file) => {
                                    let file_list = file_list.clone();
                                    files.push((
                                        hashing_file.clone(),
                                        Callback::from(move |result| {
                                            let mut lock_data = lock_data.lock().unwrap();
                                            let finished_count = lock_data.1 + 1;
                                            lock_data.1 = finished_count;
                                            let new_files = &mut lock_data.0;
                                            match result {
                                                Ok(key) => {
                                                    new_files[index].1 = File::Remote {
                                                        key: key,
                                                        name: hashing_file.file.name(),
                                                        size: hashing_file.file.size(),
                                                        mime_type: hashing_file.file.type_(),
                                                    };
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
                                File::Remote { .. } => (),
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
    let mut schema_resource_list: Vec<_> = Vec::with_capacity(edit_form.schema_resource_list.len());
    for (_key, schema_resource) in edit_form.schema_resource_list.deref().iter() {
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
    is_saving: UseStateHandle<bool>,
    err_msg: &UseStateHandle<Option<LightString>>,
    onsave: &Option<Callback<PrimaryKey>>,
) -> Result<(), LightString> {
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
                        onsave.emit(tihu::PrimaryKey { id: id });
                    }
                    None => (),
                }
                utils::success(LightString::from("保存成功"));
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
                        onsave.emit(pri_key);
                    }
                    None => (),
                }
                utils::success(LightString::from("保存成功"));
            }
        }
    }
    return Ok(());
}
