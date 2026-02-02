use super::super::extension::get_default_config;
use super::super::extension::get_parameter_schema;
use super::super::extension::parse_config;
use super::super::extension::serialize_config;
use super::super::extension::AttributeValue;
use super::super::extension::ConfigView;
use crate::components::button::Button;
use crate::components::files_upload::FilesUpload;
use crate::components::input::Input;
use crate::components::radio_group::RadioGroup;
use crate::components::required::Required;
use crate::components::rich_text::get_default_rich_text;
use crate::components::rich_text::upload_resource;
use crate::components::rich_text::RichText;
use crate::components::selection::Selection;
use crate::components::textarea::Textarea;
use crate::components::uploading_files::upload_files;
use crate::components::validate_wrapper::ValidateData;
use crate::components::validate_wrapper::ValidateWrapper;
use crate::components::visable::Visable;
use crate::components::Resource;
use crate::components::ResourceMetadata;
use crate::components::SelectOption;
use crate::sdk;
use crate::utils;
use crate::utils::gen_id;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::Key;
use crate::SharedString;
use js_sys::JSON;
use leptos::prelude::*;
use sdk::environment_schema::query_environment_schema::QueryEnvironmentSchemaApi;
use sdk::environment_schema::query_environment_schema::QueryEnvironmentSchemaReq;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema as EnvironmentSchemaDetail;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::environment_schema::read_environment_schema::SchemaResource;
use sdk::environment_schema::EnvironmentSchema;
use sdk::extension::query_extension::QueryExtensionApi;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::Attribute;
use sdk::extension::Extension;
use sdk::extension::Operation;
use sdk::job::insert_job::InsertJobApi;
use sdk::job::insert_job::InsertJobReq;
use sdk::job::read_job::Job;
use sdk::job::read_job::ReadJobApi;
use sdk::job::read_job::ReadJobReq;
use sdk::job::update_job::UpdateJobApi;
use sdk::job::update_job::UpdateJobReq;
use serde_json::Value;
use std::sync::Arc;
use std::sync::Mutex;
use tihu::Id;
use tihu::PrimaryKey;
use wasm_bindgen::prelude::*;

#[derive(Clone, PartialEq)]
struct OperationOption(Operation);

impl SelectOption for OperationOption {
    type Value = String;
    fn value(&self) -> String {
        self.0.id.clone()
    }
    fn label(&self) -> AnyView {
        self.0.name.clone().into_any()
    }
}

fn get_step_type_list() -> Vec<(StepType, String)> {
    return [StepType::Auto, StepType::Manual]
        .iter()
        .map(|value| {
            return (value.clone(), value.to_string());
        })
        .collect();
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StepType {
    Auto,   //自动
    Manual, //手动
}

impl ToString for StepType {
    fn to_string(&self) -> String {
        match *self {
            StepType::Auto => "自动".into(),
            StepType::Manual => "手动".into(),
        }
    }
}

#[derive(Clone)]
pub struct AutoStep {
    schema_resource_id: ValidateData<Option<Id>>, //环境规格资源id
    operation_id: ValidateData<Option<String>>,   //操作id
    operation_parameter: RwSignal<Vec<(Key, Attribute, AttributeValue)>>, //操作参数
}

#[derive(Clone)]
pub struct JobStep {
    step_type: RwSignal<Option<StepType>>,
    id: RwSignal<Option<Id>>,
    name: ValidateData<SharedString>,        //步骤名称
    remark: RwSignal<JsValue, LocalStorage>, //备注
    attachments: RwSignal<Vec<(Key, RwSignal<Resource, LocalStorage>, ())>, LocalStorage>, //附件
    auto_step: AutoStep,
}

#[derive(Clone)]
struct EditForm {
    active_job_step_key: RwSignal<Option<Key>>,
    environment_schema_id: ValidateData<Option<Id>>,
    name: ValidateData<SharedString>,
    remark: RwSignal<SharedString>,
    job_step_list: RwSignal<Vec<(Key, JobStep)>>,
}

#[derive(Clone)]
struct JobEditState {
    is_saving: RwSignal<bool>,
    err_msg: RwSignal<Option<SharedString>>,
    environment_schema_list: RwSignal<Vec<EnvironmentSchema>>,
    environment_schema_detail: RwSignal<Option<EnvironmentSchemaDetail>>,
    extension_list: RwSignal<Vec<Extension>>,
    edit_form: EditForm,
}

#[component]
pub fn JobEdit(
    #[prop(optional)] id: Option<Id>,
    #[prop(optional)] onsave: Option<UnsyncCallback<PrimaryKey>>,
) -> impl IntoView {
    let is_saving: RwSignal<bool> = RwSignal::new(false);
    let err_msg: RwSignal<Option<SharedString>> = RwSignal::new(None);
    let environment_schema_list: RwSignal<Vec<EnvironmentSchema>> = RwSignal::new(Vec::new());
    let environment_schema_detail: RwSignal<Option<EnvironmentSchemaDetail>> = RwSignal::new(None);
    let extension_list: RwSignal<Vec<Extension>> = RwSignal::new(Default::default());
    let edit_form = EditForm {
        active_job_step_key: RwSignal::new(Default::default()),
        environment_schema_id: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请选择环境规格"))),
        ),
        name: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请输入任务名称"))),
        ),
        remark: RwSignal::new(Default::default()),
        job_step_list: RwSignal::new(Default::default()),
    };
    let job_edit_state = JobEditState {
        is_saving: is_saving.clone(),
        err_msg: err_msg.clone(),
        environment_schema_list: environment_schema_list.clone(),
        environment_schema_detail: environment_schema_detail.clone(),
        extension_list: extension_list.clone(),
        edit_form: edit_form.clone(),
    };
    let edit_form_clone = edit_form.clone();
    let environment_schema_id = edit_form_clone.environment_schema_id.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let environment_schema_detail_clone2 = environment_schema_detail.clone();
    let environment_schema_list_clone = environment_schema_list.clone();
    let extension_list_clone = extension_list.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match query_environment_schema_list(&environment_schema_list_clone).await {
            Ok(environment_schema_list) => {
                if id.is_none() {
                    //新增场景，默认选择第一个环境规格
                    if let Some(environment_schema) = environment_schema_list.first() {
                        environment_schema_id.set(Some(environment_schema.id));
                        read_environment_schema_detail(
                            &environment_schema_detail_clone2,
                            environment_schema.id,
                        )
                        .await
                        .ok();
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
                    read_job_detail(
                        &edit_form_clone.clone(),
                        &extension_list,
                        &environment_schema_detail_clone,
                        id,
                    )
                    .await
                    .ok();
                }
            }
            Err(_err) => {
                //
            }
        }
    });
    let err_msg_clone = err_msg.clone();
    let clear_err_msg = UnsyncCallback::new(move |_: ()| {
        err_msg_clone.set(None);
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
            save_job(id, &edit_form, is_saving, &err_msg, &onsave)
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
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let job_step_list_clone = edit_form.job_step_list.clone();
    let on_environment_schema_change = UnsyncCallback::new(move |environment_schema| {
        if let Some((environment_schema_id, _)) = environment_schema {
            let environment_schema_detail = environment_schema_detail_clone.clone();
            let job_step_list = job_step_list_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                read_environment_schema_detail(&environment_schema_detail, environment_schema_id)
                    .await
                    .ok();
                //切换环境之后需要把所有步骤里面的资源和操作清除掉
                for (_, job_step) in job_step_list.get().iter() {
                    job_step.auto_step.schema_resource_id.set(None);
                    job_step.auto_step.operation_id.set(None);
                    job_step
                        .auto_step
                        .operation_parameter
                        .set(Default::default());
                }
            });
        }
    });
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let extension_list_clone = extension_list.clone();
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"任务名称："}</td>
                    <td>
                        <ValidateWrapper error={edit_form.name.error()}>
                            <Input value={edit_form.name.data()} onupdate={edit_form.name.listener()}/>
                        </ValidateWrapper>
                    </td>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"环境规格："}</td>
                    <td style="vertical-align: top;">
                        {
                            if id.is_none() {
                                view! {
                                    <ValidateWrapper error={edit_form.environment_schema_id.error()}>
                                        <Selection value={edit_form.environment_schema_id.data()} options={environment_schema_list.clone()} onchange={on_environment_schema_change.clone()}/>
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
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"备注："}</td>
                    <td colspan="3">
                        <Textarea value={edit_form.remark.clone()} onfocus={clear_err_msg.clone()} style="width:100%;"/>
                    </td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                {
                    let active_job_step_key = edit_form.active_job_step_key.clone();
                    let job_step_list = edit_form.job_step_list.clone();
                    let job_step_list_clone = edit_form.job_step_list.clone();
                    let environment_schema_detail = environment_schema_detail_clone.clone();
                    let extension_list = extension_list_clone.clone();
                    view! {
                        <div style="width:24em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                            <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"步骤列表"}</div>
                            <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                <For
                                    each={
                                        let job_step_list = edit_form.job_step_list.clone();
                                        move || { job_step_list.get().into_iter().enumerate() }
                                    }
                                    key=|(_index, (key, _job_step))| { key.clone() }
                                    children=move |(index, (key, job_step))| {
                                        let error = job_step.name.error();
                                        let job_step_list = job_step_list.clone();
                                        let name_validators = job_step.name.validators();
                                        let job_step_clone = job_step.clone();
                                        let key = key.clone();
                                        let active_job_step_key = active_job_step_key.clone();
                                        let job_edit_state = job_edit_state.clone();
                                        view! {
                                            <ValidateWrapper error={job_step.name.error()}>
                                                {
                                                    let name = job_step.name.data();
                                                    let job_step_list = job_step_list.clone();
                                                    let name_validators = name_validators.clone();
                                                    let job_step = job_step_clone.clone();
                                                    let key = key.clone();
                                                    let active_job_step_key = active_job_step_key.clone();
                                                    let job_edit_state = job_edit_state.clone();
                                                    let on_move_up = {
                                                        let job_step_list = job_step_list.clone();
                                                        UnsyncCallback::new(move |_| {
                                                            utils::move_up(&job_step_list, index);
                                                        })
                                                    };
                                                    let on_move_down = {
                                                        let job_step_list = job_step_list.clone();
                                                        UnsyncCallback::new(move |_| {
                                                            utils::move_down(&job_step_list, index);
                                                        })
                                                    };
                                                    let on_remove = {
                                                        let job_step_list = job_step_list.clone();
                                                        UnsyncCallback::new(move |_| {
                                                            utils::remove_item(&job_step_list, index);
                                                        })
                                                    };
                                                    let name_validators = name_validators.clone();
                                                    let active_job_step_key = active_job_step_key.clone();
                                                    let active_job_step_key_clone = active_job_step_key.clone();
                                                    let key = key.clone();
                                                    let is_active = {
                                                        let active_job_step_key = active_job_step_key.clone();
                                                        let key = key.clone();
                                                        move || {
                                                            &active_job_step_key.read() == &Some(key.clone())
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
                                                                    let active_job_step_key = active_job_step_key_clone.clone();
                                                                    let key = key.clone();
                                                                    wasm_bindgen_futures::spawn_local(async move {
                                                                        active_job_step_key.set(Some(key.clone()));
                                                                        utils::wait(0).await;
                                                                        utils::trigger_resize();
                                                                    });
                                                                }} style="flex-grow: 1;flex-shrink: 1;padding: 0.5em 0;">
                                                                    {
                                                                        move || {
                                                                            let name = name.get();
                                                                            if name.is_empty() {
                                                                                SharedString::from("(缺少步骤名称)")
                                                                            } else {
                                                                                name
                                                                            }
                                                                        }
                                                                    }
                                                                </div>
                                                                <Button disabled={0==index} onclick={on_move_up} style={SharedString::from("margin-left:0.5em;")}>{"上移"}</Button>
                                                                <Button disabled={move || index + 1 == job_step_list.read().len()} onclick={on_move_down} style={SharedString::from("margin-left:0.5em;")}>{"下移"}</Button>
                                                                <Button onclick={on_remove} style={SharedString::from("margin-left:0.5em;")}>{"移除"}</Button>
                                                            </div>
                                                            <Visable condition={is_active} style="position:absolute;left:24em;right:0;top:0;bottom:0;overflow: auto;">
                                                                {job_edit_state.job_step_edit_view(&job_step, name.clone(), error, name_validators)}
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
                                        let resource_list = get_resource_list(&environment_schema_detail.read());
                                        let (schema_resource_id, operation_id, operation_parameter) = if let Some(resource) = resource_list.first() {
                                            let operations = get_operations(&extension_list.read(), &resource.extension_id).map(|v|v.clone()).unwrap_or_default();
                                            if let Some(operation) = operations.first() {
                                                (Some(resource.id), Some(operation.id.clone()), get_default_config(operation.parameter_schema.clone()))
                                            } else {
                                                (Some(resource.id), None, Vec::new())
                                            }
                                        } else {
                                            (None, None, Vec::new())
                                        };
                                        let new_job_step = JobStep {
                                            step_type: RwSignal::new(Some(StepType::Auto)),
                                            id: Default::default(),
                                            name: init_step_name(Default::default()),
                                            remark: RwSignal::new_local(get_default_rich_text()),
                                            attachments: Default::default(),
                                            auto_step: AutoStep {
                                                schema_resource_id: init_schema_resource_id(schema_resource_id),
                                                operation_id: init_operation_id(operation_id),
                                                operation_parameter: RwSignal::new(operation_parameter),
                                            },
                                        };
                                        let new_key: Key = gen_id().into();
                                        active_job_step_key.set(Some(new_key.clone()));
                                        job_step_list_clone.write().push((new_key, new_job_step));
                                    })}>{"添加"}</Button>
                                </div>
                            </div>
                        </div>
                    }
                }
            </div>
            <div style="padding-top:0.25em;">
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

impl JobEditState {
    fn job_step_edit_view(
        &self,
        job_step: &JobStep,
        name: RwSignal<SharedString>,
        error: RwSignal<Option<SharedString>>,
        name_validators: Arc<Validators<SharedString>>,
    ) -> impl IntoView + use<> {
        let extension_list = self.extension_list.clone();
        let environment_schema_detail = self.environment_schema_detail.clone();
        let err_msg = self.err_msg.clone();
        let clear_err_msg = UnsyncCallback::new(move |_| {
            err_msg.set(None);
        });
        let auto_step = job_step.auto_step.clone();
        let attachments = job_step.attachments.clone();
        view! {
            <table style="width: 100%;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="vertical-align: top; width:6em;"><Required/>{"步骤名称："}</td>
                    <td>
                        <ValidateWrapper error={error.clone()} style="display:inline-block;">
                            <Input value={name} onupdate={UnsyncCallback::new(move |value| {
                                name_validators.validate_into(&value, &error)
                            })}/>
                        </ValidateWrapper>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="vertical-align: top;">{"备注："}</td>
                    <td>
                        <RichText value={job_step.remark.clone()} style="border: 1px solid rgba(0, 0, 0, 0.2);padding: 0.25em 0;min-height: 8em;"/>

                    </td>
                </tr>
                <tr>
                    <td class="align-right"><Required/>{"步骤类型："}</td>
                    <td>
                        <RadioGroup value={job_step.step_type.clone()} options={get_step_type_list()} onchange={clear_err_msg} />
                    </td>
                </tr>
                {
                    let step_type = job_step.step_type.clone();
                    move || {
                        if let Some(step_type) = step_type.get() {
                            match step_type {
                                StepType::Auto => {
                                    let extension_list = extension_list.clone();
                                    let schema_resource_id = auto_step.schema_resource_id.clone();
                                    let operation_id = auto_step.operation_id.clone();
                                    let operation_parameter = auto_step.operation_parameter.clone();
                                    let environment_schema_detail = environment_schema_detail.clone();
                                    view! {
                                        <ValidateWrapper error={schema_resource_id.error()}>
                                            {
                                                let schema_resource_list = {
                                                    let environment_schema_detail = environment_schema_detail.clone();
                                                    Signal::derive(move || {
                                                        environment_schema_detail.read().as_ref().map(|detail| {
                                                            detail.resource_list.iter().map(|resource| {
                                                                (resource.id, resource.name.clone())
                                                            }).collect()
                                                        }).unwrap_or_default()
                                                    })
                                                };
                                                let extension_list = extension_list.clone();
                                                let extension_list_clone = extension_list.clone();
                                                let environment_schema_detail = environment_schema_detail.clone();
                                                let environment_schema_detail_clone = environment_schema_detail.clone();
                                                let operation_parameter = operation_parameter.clone();
                                                let operation_parameter_clone = operation_parameter.clone();
                                                let schema_resource_id_clone = schema_resource_id.clone();
                                                let operation_id = operation_id.clone();
                                                let operation_id_clone = operation_id.clone();
                                                let on_schema_resource_change = UnsyncCallback::new(move |schema_resource| {
                                                    if let Some((schema_resource_id, _)) = schema_resource {
                                                        let extension_id = get_extension_id(&environment_schema_detail_clone, schema_resource_id);
                                                        let operations: Vec<OperationOption> = extension_id.map(|extension_id| get_operations(
                                                            &extension_list_clone.read(),
                                                            &extension_id,
                                                        ).map(|operations| {
                                                            operations.iter().map(|operation| {
                                                                OperationOption(operation.clone())
                                                            }).collect()
                                                        })).flatten().unwrap_or_default();
                                                        if let Some(operation) = operations.first() {
                                                            operation_id_clone.set(Some(operation.0.id.clone()));
                                                            let default_parameter = get_default_config(operation.0.parameter_schema.clone());
                                                            operation_parameter.set(default_parameter);
                                                        } else {
                                                            //把对应的操作清除掉
                                                            operation_id_clone.set(None);
                                                            operation_parameter.set(Default::default());
                                                        }
                                                    }
                                                });
                                                view! {
                                                    <>
                                                        <tr>
                                                            <td class="align-right"><Required/>{"操作资源："}</td>
                                                            <td>
                                                                <Selection value={schema_resource_id.data()} options={schema_resource_list} onchange={on_schema_resource_change}/>
                                                            </td>
                                                        </tr>
                                                        <tr>
                                                            <td class="align-right" style="vertical-align: top;"><Required/>{"操作类型："}</td>
                                                            <td>
                                                                <ValidateWrapper error={operation_id.error()}>
                                                                    {
                                                                        let operations = {
                                                                            let extension_list = extension_list.clone();
                                                                            let schema_resource_id = schema_resource_id_clone.clone();
                                                                            Signal::derive(move || {
                                                                                let extension_id = schema_resource_id.get().map(|schema_resource_id| {
                                                                                    get_extension_id(
                                                                                        &environment_schema_detail,
                                                                                        schema_resource_id,
                                                                                    )
                                                                                }).flatten();
                                                                                extension_id.map(|extension_id| get_operations(
                                                                                    &extension_list.read(),
                                                                                    &extension_id,
                                                                                ).map(|operations| {
                                                                                    operations.iter().map(|operation| {
                                                                                        OperationOption(operation.clone())
                                                                                    }).collect()
                                                                                })).flatten().unwrap_or_default()
                                                                            })
                                                                        };
                                                                        let on_operation_change = {
                                                                            let extension_list = extension_list.clone();
                                                                            let schema_resource_id = schema_resource_id_clone.clone();
                                                                            let operation_parameter = operation_parameter_clone.clone();
                                                                            let environment_schema_detail = environment_schema_detail.clone();
                                                                            UnsyncCallback::new(move |operation: Option<OperationOption>| {
                                                                                if let Some(operation) = operation {
                                                                                    let extension_id = schema_resource_id.get().map(|schema_resource_id| {
                                                                                        get_extension_id(
                                                                                            &environment_schema_detail,
                                                                                            schema_resource_id,
                                                                                        )
                                                                                    }).flatten();
                                                                                    let default_parameter = extension_id.map(|extension_id| get_operations(
                                                                                        &extension_list.read(),
                                                                                        &extension_id,
                                                                                    ).map(|operations| {
                                                                                        operations.iter().find(|item| {
                                                                                            item.id == operation.0.id
                                                                                        }).map(|operation| get_default_config(operation.parameter_schema.clone()))
                                                                                    })).flatten().flatten().unwrap_or_default();
                                                                                    operation_parameter.set(default_parameter);
                                                                                }
                                                                            })
                                                                        };
                                                                        view! {
                                                                            <Selection value={operation_id.data()} options={operations.clone()} onchange={on_operation_change}/>
                                                                        }
                                                                    }
                                                                </ValidateWrapper>
                                                            </td>
                                                        </tr>
                                                        <tr>
                                                            <td class="align-right" style="vertical-align: top;">{"操作参数："}</td>
                                                            <td>
                                                                <ConfigView attributes={operation_parameter}/>
                                                            </td>
                                                        </tr>
                                                    </>
                                                }
                                            }
                                        </ValidateWrapper>
                                    }.into_any()
                                },
                                StepType::Manual => view! {
                                    <tr>
                                        <td class="align-right" style="vertical-align: top;">{"附件："}</td>
                                        <td>
                                            <FilesUpload files={attachments}/>
                                        </td>
                                    </tr>
                                }.into_any()
                            }
                        } else {
                            view! {}.into_any()
                        }
                    }
                }
            </table>
        }
    }
}

fn init_step_name(value: SharedString) -> ValidateData<SharedString> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请输入步骤名称"))),
    )
}

fn init_schema_resource_id(value: Option<Id>) -> ValidateData<Option<Id>> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请选择资源规格"))),
    )
}

fn init_operation_id(value: Option<String>) -> ValidateData<Option<String>> {
    ValidateData::new(
        value,
        Some(Validators::new().add(RequiredValidator::new("请选择操作类型"))),
    )
}

async fn query_extension_list(
    extension_list: &RwSignal<Vec<Extension>>,
) -> Result<Vec<Extension>, SharedString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
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

async fn read_job_detail(
    edit_form: &EditForm,
    extension_list: &[Extension],
    environment_schema_detail: &RwSignal<Option<EnvironmentSchemaDetail>>,
    id: Id,
) -> Result<Job, SharedString> {
    let params = ReadJobReq { id: id };
    let mut job = ReadJobApi.call(&params).await?;
    job.job_step_list.sort_by_key(|job_step| match job_step {
        sdk::job::read_job::JobStep::Auto { seq, .. } => *seq,
        sdk::job::read_job::JobStep::Manual { seq, .. } => *seq,
    });
    edit_form
        .environment_schema_id
        .set(job.environment_schema_id.into());
    edit_form.name.set(job.name.clone().into());
    edit_form
        .remark
        .set(job.remark.clone().unwrap_or_default().into());
    let environment_schema_detail =
        read_environment_schema_detail(environment_schema_detail, job.environment_schema_id)
            .await?;
    let resource_list = environment_schema_detail.resource_list;
    edit_form.job_step_list.set(
        job.job_step_list
            .iter()
            .map(|job_step| {
                (
                    utils::gen_id().into(),
                    match job_step {
                        sdk::job::read_job::JobStep::Auto {
                            id,
                            name,
                            schema_resource_id,
                            operation_id,
                            operation_parameter,
                            remark,
                            ..
                        } => {
                            let operation_parameter =
                                resource_list
                                    .iter()
                                    .find(|resource| &resource.id == schema_resource_id)
                                    .map(|resource| {
                                        extension_list
                                            .iter()
                                            .find(|extension| extension.id == resource.extension_id)
                                            .map(|extension| {
                                                get_parameter_schema(
                                                    &extension.operations,
                                                    &operation_id,
                                                )
                                                .map(|parameter_schema| {
                                                    parse_config(
                                                        parameter_schema.clone(),
                                                        &operation_parameter,
                                                    )
                                                })
                                            })
                                    })
                                    .flatten()
                                    .flatten()
                                    .unwrap_or_default();
                            JobStep {
                                step_type: RwSignal::new(Some(StepType::Auto)),
                                id: RwSignal::new(Some(*id)),
                                name: init_step_name(name.clone().into()),
                                remark: RwSignal::new_local(
                                    remark
                                        .clone()
                                        .map(|remark| {
                                            JSON::parse(&remark).unwrap_or_else(|err| {
                                                log::error!("备注格式不正确: {:?}", err);
                                                get_default_rich_text()
                                            })
                                        })
                                        .unwrap_or_else(get_default_rich_text),
                                ),
                                attachments: Default::default(),
                                auto_step: AutoStep {
                                    schema_resource_id: init_schema_resource_id(Some(
                                        *schema_resource_id,
                                    )),
                                    operation_id: init_operation_id(operation_id.clone().into()),
                                    operation_parameter: RwSignal::new(operation_parameter),
                                },
                            }
                        }
                        sdk::job::read_job::JobStep::Manual {
                            id,
                            name,
                            remark,
                            attachments,
                            ..
                        } => JobStep {
                            step_type: RwSignal::new(Some(StepType::Manual)),
                            id: RwSignal::new(Some(*id)),
                            name: init_step_name(name.clone().into()),
                            remark: RwSignal::new_local(
                                remark
                                    .clone()
                                    .map(|remark| {
                                        JSON::parse(&remark).unwrap_or_else(|err| {
                                            log::error!("备注格式不正确: {:?}", err);
                                            get_default_rich_text()
                                        })
                                    })
                                    .unwrap_or_else(get_default_rich_text),
                            ),
                            attachments: RwSignal::new_local(
                                attachments
                                    .as_ref()
                                    .map(|attachments| {
                                        serde_json::from_str::<serde_json::Value>(attachments)
                                            .map(|value| {
                                                value
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
                                                                            .map(|value| {
                                                                                value.to_string()
                                                                            })
                                                                            .unwrap_or_default();
                                                                        let name = map
                                                                            .get("name")
                                                                            .unwrap()
                                                                            .as_str()
                                                                            .map(|value| {
                                                                                value.to_string()
                                                                            })
                                                                            .unwrap_or_default();
                                                                        let size = map
                                                                            .get("size")
                                                                            .unwrap()
                                                                            .as_f64()
                                                                            .unwrap();
                                                                        let mime_type = map
                                                                            .get("mime_type")
                                                                            .unwrap()
                                                                            .as_str()
                                                                            .map(|value| {
                                                                                value.to_string()
                                                                            })
                                                                            .unwrap_or_default();
                                                                        let file = Resource::Remote(
                                                                            ResourceMetadata {
                                                                                key: key,
                                                                                name: name,
                                                                                size: size,
                                                                                mime_type:
                                                                                    mime_type,
                                                                            },
                                                                        );
                                                                        (
                                                                            gen_id().into(),
                                                                            RwSignal::new_local(
                                                                                file,
                                                                            ),
                                                                            (),
                                                                        )
                                                                    })
                                                                    .unwrap()
                                                            })
                                                            .collect()
                                                    })
                                                    .unwrap_or_default()
                                            })
                                            .unwrap_or_default()
                                    })
                                    .unwrap_or_default(),
                            ),
                            auto_step: AutoStep {
                                schema_resource_id: Default::default(),
                                operation_id: Default::default(),
                                operation_parameter: Default::default(),
                            },
                        },
                    },
                )
            })
            .collect(),
    );
    return Ok(job);
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

fn get_resource_list(detail: &Option<EnvironmentSchemaDetail>) -> Vec<SchemaResource> {
    detail
        .as_ref()
        .map(|detail| detail.resource_list.clone())
        .unwrap_or_default()
}

fn get_operations<'a>(
    extension_list: &'a [Extension],
    extension_id: &str,
) -> Option<&'a Vec<Operation>> {
    return extension_list
        .iter()
        .find(|extension| extension.id == extension_id)
        .map(|extension| &extension.operations);
}

fn get_extension_id(
    environment_schema_detail: &RwSignal<Option<EnvironmentSchemaDetail>>,
    schema_resource_id: Id,
) -> Option<String> {
    return environment_schema_detail
        .read()
        .as_ref()
        .map(|detail| {
            detail
                .resource_list
                .iter()
                .find(|resource| resource.id == schema_resource_id)
                .map(|item| item.extension_id.clone())
        })
        .flatten();
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
    let mut active_step_key = None;
    for (step_key, job_step) in edit_form.job_step_list.get().iter() {
        if let Err(error) = job_step.name.validate(true) {
            err_msgs.push(error);
            if active_step_key.is_none() {
                active_step_key.replace(step_key.clone());
            }
        }
        let step_type = job_step.step_type.get();
        if let Some(step_type) = step_type {
            if StepType::Auto == step_type {
                if let Err(error) = job_step.auto_step.schema_resource_id.validate(true) {
                    err_msgs.push(error);
                    if active_step_key.is_none() {
                        active_step_key.replace(step_key.clone());
                    }
                }
                if let Err(error) = job_step.auto_step.operation_id.validate(true) {
                    err_msgs.push(error);
                    if active_step_key.is_none() {
                        active_step_key.replace(step_key.clone());
                    }
                }
                for (_, _, value) in job_step.auto_step.operation_parameter.get().iter() {
                    if let Err(error) = value.validate(true) {
                        err_msgs.push(error);
                        if active_step_key.is_none() {
                            active_step_key.replace(step_key.clone());
                        }
                    }
                }
            }
        }
    }
    if let Some(active_step_key) = active_step_key {
        edit_form.active_job_step_key.set(Some(active_step_key));
        utils::wait(0).await;
        utils::trigger_resize();
    }
    return err_msgs;
}

async fn try_upload_files(edit_form: &EditForm) -> Result<(), SharedString> {
    let mut files = Vec::new();
    for (_, job_step) in edit_form.job_step_list.get().iter() {
        let remark = job_step.remark.get();
        upload_resource(&remark).await.map_err(|err| {
            log::error!("上传富文本资源失败: {:?}", err);
            return SharedString::from("上传文件失败");
        })?;
        let step_type = job_step.step_type.get();
        if let Some(step_type) = step_type {
            match step_type {
                StepType::Auto => {
                    let operation_parameter = job_step.auto_step.operation_parameter.get();
                    for (_, _, attr_value) in operation_parameter {
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
                                                        handle
                                                            .set(Some(Resource::Remote(metadata)));
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
                                for (index, (_key, file, ())) in
                                    file_list.get().into_iter().enumerate()
                                {
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
                StepType::Manual => {
                    let new_files = job_step.attachments.get();
                    let file_count = new_files.len();
                    let finished_count = 0;
                    let lock_data = Arc::new(Mutex::new((new_files, finished_count)));
                    for (index, (_key, file, ())) in
                        job_step.attachments.get().into_iter().enumerate()
                    {
                        let lock_data = lock_data.clone();
                        match file.get() {
                            Resource::Local(hashing_file) => {
                                let attachments = job_step.attachments.clone();
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
                                            attachments.set(new_files.clone());
                                        }
                                    }),
                                ));
                            }
                            Resource::Remote(_metadata) => (),
                        }
                    }
                }
            }
        }
    }
    upload_files(files).await?;
    //延迟0秒，让修改的值生效
    utils::wait(0).await;
    return Ok(());
}

fn collect_job_step_list_list(
    edit_form: &EditForm,
) -> Vec<(
    Option<Id>,
    String,
    Option<String>,
    Option<String>,
    Option<(Id, String, String)>,
)> {
    let list = edit_form.job_step_list.get();
    let mut job_step_list: Vec<_> = Vec::with_capacity(list.len());
    for (_, job_step) in list.iter() {
        let step_type = job_step.step_type.get();
        let id = job_step.id.get();
        let name = job_step.name.get();
        let remark = job_step.remark.get();
        let remark = Some(JSON::stringify(&remark).unwrap().as_string().unwrap());
        if let Some(step_type) = step_type {
            match step_type {
                StepType::Auto => {
                    let schema_resource_id = job_step.auto_step.schema_resource_id.get();
                    let operation_id = job_step.auto_step.operation_id.get();
                    let operation_parameter =
                        serialize_config(&job_step.auto_step.operation_parameter.get());
                    job_step_list.push((
                        id,
                        name.to_string(),
                        remark,
                        None,
                        Some((
                            schema_resource_id.unwrap(),
                            operation_id.unwrap(),
                            operation_parameter,
                        )),
                    ));
                }
                StepType::Manual => {
                    let files: Vec<Value> = job_step
                        .attachments
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
                        .collect();
                    let attachments = if files.is_empty() {
                        None
                    } else {
                        serde_json::to_string(&Value::Array(files)).ok()
                    };
                    job_step_list.push((id, name.to_string(), remark, attachments, None));
                }
            }
        }
    }
    return job_step_list;
}

async fn save_job(
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
    let job_step_list = collect_job_step_list_list(edit_form);
    let name = edit_form.name.get().to_string();
    let remark = edit_form.remark.get();
    let remark = if remark.is_empty() {
        None
    } else {
        Some(remark.to_string())
    };
    if let Some(id) = id {
        let params = UpdateJobReq {
            id: id,
            name: name,
            remark: remark,
            job_step_list: job_step_list
                .into_iter()
                .enumerate()
                .map(|(index, (id, name, remark, attachments, auto_step))| {
                    if let Some((schema_resource_id, operation_id, operation_parameter)) = auto_step
                    {
                        sdk::job::update_job::JobStep::Auto {
                            id: id,
                            name: name,
                            schema_resource_id: schema_resource_id,
                            operation_id: operation_id,
                            operation_parameter: operation_parameter,
                            remark: remark,
                            seq: index as i32,
                        }
                    } else {
                        sdk::job::update_job::JobStep::Manual {
                            id: id,
                            name: name,
                            remark: remark,
                            attachments: attachments,
                            seq: index as i32,
                        }
                    }
                })
                .collect(),
        };
        let ret = UpdateJobApi.lock_handler(is_saving).call(&params).await;
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
        let params = InsertJobReq {
            environment_schema_id: edit_form.environment_schema_id.get().unwrap(),
            name: name,
            remark: remark,
            job_step_list: job_step_list
                .into_iter()
                .enumerate()
                .map(|(index, (_id, name, remark, attachments, auto_step))| {
                    if let Some((schema_resource_id, operation_id, operation_parameter)) = auto_step
                    {
                        sdk::job::insert_job::JobStep::Auto {
                            name: name,
                            schema_resource_id: schema_resource_id,
                            operation_id: operation_id,
                            operation_parameter: operation_parameter,
                            remark: remark,
                            seq: index as i32,
                        }
                    } else {
                        sdk::job::insert_job::JobStep::Manual {
                            name: name,
                            remark: remark,
                            attachments: attachments,
                            seq: index as i32,
                        }
                    }
                })
                .collect(),
        };
        let ret = InsertJobApi.lock_handler(is_saving).call(&params).await;
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
