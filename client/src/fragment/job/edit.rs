use super::super::extension::config_view;
use super::super::extension::get_default_config;
use super::super::extension::get_parameter_schema;
use super::super::extension::parse_config;
use super::super::extension::serialize_config;
use super::super::extension::AttributeValue;
use crate::components::button::Button;
use crate::components::files_upload::BindingFilesUpload;
use crate::components::input::BindingInput;
use crate::components::r#if::If;
use crate::components::radio_group::BindingRadioGroup;
use crate::components::required::Required;
use crate::components::rich_text::get_default_rich_text;
use crate::components::rich_text::upload_resource;
use crate::components::rich_text::BindingRichText;
use crate::components::selection::BindingSelection;
use crate::components::show::Show;
use crate::components::textarea::BindingTextarea;
use crate::components::uploading_files::upload_files;
use crate::components::validate_wrapper::ValidateData;
use crate::components::validate_wrapper::ValidateWrapper;
use crate::components::File;
use crate::components::SelectOption;
use crate::sdk;
use crate::utils;
use crate::utils::binding::Binding;
use crate::utils::gen_id;
use crate::utils::request::ApiExt;
use crate::utils::validator::RequiredValidator;
use crate::utils::validator::Validators;
use crate::LightString;
use js_sys::JSON;
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
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use tihu::Id;
use tihu::PrimaryKey;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::virtual_dom::Key;

type EnvironmentSchemaSelection = BindingSelection<(Id, String)>;
type StepTypeRadioGroup = BindingRadioGroup<(StepType, String)>;
type SchemaResourceSelection = BindingSelection<(Id, String)>;
#[derive(Clone, PartialEq)]
struct OperationOption(Operation);
type OperationSelection = BindingSelection<OperationOption>;

impl SelectOption for OperationOption {
    type Value = String;
    fn value(&self) -> String {
        self.0.id.clone()
    }
    fn label(&self) -> yew::Html {
        yew::Html::from(self.0.name.to_string())
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

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct AutoStep {
    schema_resource_id: ValidateData<Option<Id>>, //环境规格资源id
    operation_id: ValidateData<Option<String>>,   //操作id
    operation_parameter: Binding<Vec<(Key, Attribute, AttributeValue)>>, //操作参数
}

#[derive(Clone, PartialEq, Debug)]
pub struct JobStep {
    step_type: Binding<StepType>,
    id: Binding<Option<Id>>,
    name: ValidateData<LightString>,            //步骤名称
    remark: Binding<JsValue>,                   //备注
    attachments: Binding<Vec<(Key, File, ())>>, //附件
    auto_step: AutoStep,
}

#[derive(Clone)]
struct EditForm {
    active_job_step_key: UseStateHandle<Option<Key>>,
    environment_schema_id: ValidateData<Option<Id>>,
    name: ValidateData<LightString>,
    remark: UseStateHandle<LightString>,
    job_step_list: UseStateHandle<Vec<(Key, JobStep)>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Option<Id>,
    #[prop_or_default]
    pub onsave: Option<Callback<PrimaryKey>>,
}

#[derive(Clone)]
struct JobEditState {
    is_saving: UseStateHandle<bool>,
    err_msg: UseStateHandle<Option<LightString>>,
    environment_schema_list: UseStateHandle<Vec<EnvironmentSchema>>,
    environment_schema_detail: UseStateHandle<Option<EnvironmentSchemaDetail>>,
    extension_list: UseStateHandle<Vec<Extension>>,
    edit_form: EditForm,
}

#[function_component]
pub fn JobEdit(props: &Props) -> Html {
    let is_saving: UseStateHandle<bool> = use_state(|| false);
    let err_msg: UseStateHandle<Option<LightString>> = use_state(|| None);
    let environment_schema_list: UseStateHandle<Vec<EnvironmentSchema>> = use_state(|| Vec::new());
    let environment_schema_detail: UseStateHandle<Option<EnvironmentSchemaDetail>> =
        use_state(|| None);
    let extension_list: UseStateHandle<Vec<Extension>> = use_state(|| Default::default());
    let edit_form = EditForm {
        active_job_step_key: use_state(|| Default::default()),
        environment_schema_id: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请选择环境规格"))),
        ),
        name: ValidateData::new(
            Default::default(),
            Some(Validators::new().add(RequiredValidator::new("请输入任务名称"))),
        ),
        remark: use_state(|| Default::default()),
        job_step_list: use_state(|| Default::default()),
    };
    let job_edit_state = JobEditState {
        is_saving: is_saving.clone(),
        err_msg: err_msg.clone(),
        environment_schema_list: environment_schema_list.clone(),
        environment_schema_detail: environment_schema_detail.clone(),
        extension_list: extension_list.clone(),
        edit_form: edit_form.clone(),
    };
    let id = props.id;
    let edit_form_clone = edit_form.clone();
    let environment_schema_id = edit_form_clone.environment_schema_id.clone();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let environment_schema_detail_clone2 = environment_schema_detail.clone();
    let environment_schema_list_clone = environment_schema_list.clone();
    let extension_list_clone = extension_list.clone();
    use_effect_with(id, move |_| {
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
            save_job(id, &edit_form, is_saving, &err_msg, &onsave)
                .await
                .ok();
        });
    });
    let environment_schema_list: Vec<_> = environment_schema_list
        .iter()
        .map(|item| (item.id.clone().into(), item.name.clone()))
        .collect();
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let job_step_list_clone = edit_form.job_step_list.clone();
    let on_environment_schema_change = Callback::from(move |environment_schema| {
        if let Some((environment_schema_id, _)) = environment_schema {
            let environment_schema_detail = environment_schema_detail_clone.clone();
            let job_step_list = job_step_list_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                read_environment_schema_detail(&environment_schema_detail, environment_schema_id)
                    .await
                    .ok();
                //切换环境之后需要把所有步骤里面的资源和操作清除掉
                for (_, job_step) in job_step_list.iter() {
                    if let Some(schema_resource_id) =
                        job_step.auto_step.schema_resource_id.get_state()
                    {
                        schema_resource_id.set(None);
                    }
                    if let Some(operation_id) = job_step.auto_step.operation_id.get_state() {
                        operation_id.set(None);
                    }
                    if let Some(operation_parameter) =
                        job_step.auto_step.operation_parameter.get_state()
                    {
                        operation_parameter.set(Default::default());
                    }
                }
            });
        }
    });
    let environment_schema_detail_clone = environment_schema_detail.clone();
    let extension_list_clone = extension_list.clone();
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;display:flex;flex-direction: column;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;"><Required/>{"任务名称："}</td>
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
                                html! {
                                    edit_form.environment_schema_id.view(move |environment_schema_id, _validator| {
                                        html! {
                                            <EnvironmentSchemaSelection value={environment_schema_id} options={environment_schema_list.clone()} onchange={on_environment_schema_change.clone()}/>
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
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"备注："}</td>
                    <td colspan="3">
                        <BindingTextarea value={edit_form.remark.clone()} onfocus={clear_err_msg.clone()} style="width:100%;"/>
                    </td>
                </tr>
            </table>
            <div style="flex-grow: 1;flex-shrink: 1;position: relative;border-top: 1px solid #CCC;border-bottom: 1px solid #CCC;overflow: auto;">
                {
                    {
                        let active_job_step_key = edit_form.active_job_step_key.clone();
                        let job_step_list = edit_form.job_step_list.clone();
                        let job_step_list_clone = edit_form.job_step_list.clone();
                        let environment_schema_detail = environment_schema_detail_clone.clone();
                        let extension_list = extension_list_clone.clone();
                        html! {
                            <div style="width:24em;height:100%;display:flex;flex-direction:column;border-right: 1px solid #CCC;box-sizing: border-box;">
                                <div style="font-weight: bold;border-bottom: 1px solid #CCC;padding-bottom: 0.5em;">{"步骤列表"}</div>
                                <div style="flex-grow: 1;flex-shrink: 1;overflow: auto;">
                                    {
                                        html! {
                                            for edit_form.job_step_list.deref().clone().into_iter().enumerate().map(|(index, (key, job_step))| {
                                                let error = job_step.name.error.clone();
                                                let job_step_list = job_step_list.clone();
                                                let name_validators = job_step.name.validators.clone();
                                                let job_step_clone = job_step.clone();
                                                let key = key.clone();
                                                let active_job_step_key = active_job_step_key.clone();
                                                let job_edit_state = job_edit_state.clone();
                                                html! {
                                                    <div key={key.clone()}>
                                                        {
                                                            job_step.name.data.view(move |name: UseStateHandle<AttrValue>| {
                                                                let name = name.clone();
                                                                let job_step_list = job_step_list.clone();
                                                                let name_validators = name_validators.clone();
                                                                let job_step = job_step_clone.clone();
                                                                let key = key.clone();
                                                                let active_job_step_key = active_job_step_key.clone();
                                                                let job_edit_state = job_edit_state.clone();
                                                                error.view(move |error: UseStateHandle<Option<AttrValue>>| {
                                                                    let on_move_up = {
                                                                        let job_step_list = job_step_list.clone();
                                                                        Callback::from(move |_| {
                                                                            utils::move_up(&job_step_list, index);
                                                                        })
                                                                    };
                                                                    let on_move_down = {
                                                                        let job_step_list = job_step_list.clone();
                                                                        Callback::from(move |_| {
                                                                            utils::move_down(&job_step_list, index);
                                                                        })
                                                                    };
                                                                    let on_remove = {
                                                                        let job_step_list = job_step_list.clone();
                                                                        Callback::from(move |_| {
                                                                            utils::remove_item(&job_step_list, index);
                                                                        })
                                                                    };
                                                                    let name_validators = name_validators.clone();
                                                                    let active_job_step_key = active_job_step_key.clone();
                                                                    let active_job_step_key_clone = active_job_step_key.clone();
                                                                    let key = key.clone();
                                                                    let key_clone = key.clone();
                                                                    let is_active = active_job_step_key.deref() == &Some(key_clone);
                                                                    let background_color = if is_active {
                                                                        "background-color: #EEE"
                                                                    } else {
                                                                        ""
                                                                    };
                                                                    html! {
                                                                        <div>
                                                                            <div style={format!("border-bottom: 1px solid #CCC;padding: 0 0.5em;display: flex;justify-content: space-between;align-items: center;{}", background_color)}>
                                                                                <div onclick={Callback::from(move |_| {
                                                                                    let active_job_step_key = active_job_step_key_clone.clone();
                                                                                    let key = key.clone();
                                                                                    wasm_bindgen_futures::spawn_local(async move {
                                                                                        active_job_step_key.set(Some(key.clone()));
                                                                                        utils::wait(0).await;
                                                                                        utils::trigger_resize();
                                                                                    });
                                                                                })} style="flex-grow: 1;flex-shrink: 1;padding: 0.5em 0;">
                                                                                    {
                                                                                        if name.is_empty() {
                                                                                            LightString::from("(缺少步骤名称)")
                                                                                        } else {
                                                                                            name.deref().clone()
                                                                                        }
                                                                                    }
                                                                                </div>
                                                                                <Button disabled={0==index} onclick={on_move_up} style="margin-left:0.5em;">{"上移"}</Button>
                                                                                <Button disabled={index + 1 == job_step_list.len()} onclick={on_move_down} style="margin-left:0.5em;">{"下移"}</Button>
                                                                                <Button onclick={on_remove} style="margin-left:0.5em;">{"移除"}</Button>
                                                                            </div>
                                                                            <Show condition={is_active} style="position:absolute;left:24em;right:0;top:0;bottom:0;overflow: auto;">
                                                                                {job_edit_state.job_step_edit_view(&job_step, name.clone(), error, name_validators)}
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
                                    }
                                    <div style="margin-top: 0.5em;">
                                        <Button onclick={Callback::from(move |_| {
                                            let resource_list = get_resource_list(environment_schema_detail.deref());
                                            let (schema_resource_id, operation_id, operation_parameter) = if let Some(resource) = resource_list.first() {
                                                let operations = get_operations(&extension_list, &resource.extension_id).map(|v|v.clone()).unwrap_or_default();
                                                if let Some(operation) = operations.first() {
                                                    (Some(resource.id), Some(operation.id.clone()), get_default_config(operation.parameter_schema.clone()))
                                                } else {
                                                    (Some(resource.id), None, Vec::new())
                                                }
                                            } else {
                                                (None, None, Vec::new())
                                            };
                                            let new_job_step = JobStep {
                                                step_type: Binding::new(StepType::Auto),
                                                id: Default::default(),
                                                name: init_step_name(Default::default()),
                                                remark: Binding::new(get_default_rich_text()),
                                                attachments: Default::default(),
                                                auto_step: AutoStep {
                                                    schema_resource_id: init_schema_resource_id(schema_resource_id),
                                                    operation_id: init_operation_id(operation_id),
                                                    operation_parameter: Binding::new(operation_parameter),
                                                },
                                            };
                                            let mut new_list = job_step_list_clone.deref().clone();
                                            let new_key: Key = gen_id().into();
                                            active_job_step_key.set(Some(new_key.clone()));
                                            new_list.push((new_key, new_job_step));
                                            job_step_list_clone.set(new_list);
                                        })}>{"添加"}</Button>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                }
            </div>
            <div style="padding-top:0.25em;">
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

impl JobEditState {
    fn job_step_edit_view(
        &self,
        job_step: &JobStep,
        name: UseStateHandle<LightString>,
        error: UseStateHandle<Option<LightString>>,
        name_validators: Validators<LightString>,
    ) -> Html {
        let extension_list = self.extension_list.clone();
        let environment_schema_detail = self.environment_schema_detail.clone();
        let err_msg = self.err_msg.clone();
        let clear_err_msg = Callback::from(move |_: ()| {
            err_msg.set(None);
        });
        let auto_step = job_step.auto_step.clone();
        let attachments = job_step.attachments.clone();
        html! {
            <table style="width: 100%;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="vertical-align: top; width:6em;"><Required/>{"步骤名称："}</td>
                    <td>
                        <ValidateWrapper error={error.deref().clone()} style="display:inline-block;">
                            <BindingInput value={name} onupdate={Callback::from(move |value| {
                                name_validators.validate_into(&value, &error)
                            })}/>
                        </ValidateWrapper>
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="vertical-align: top;">{"备注："}</td>
                    <td>
                        {
                            job_step.remark.view(move |remark| {
                                html! {
                                    <BindingRichText value={remark} style="border: 1px solid rgba(0, 0, 0, 0.2);padding: 0.25em 0;min-height: 8em;"/>
                                }
                            })
                        }
                    </td>
                </tr>
                {
                    job_step.step_type.view(move |step_type: UseStateHandle<StepType>| {
                        html! {
                            <>
                                <tr>
                                    <td class="align-right"><Required/>{"步骤类型："}</td>
                                    <td>
                                        <StepTypeRadioGroup value={step_type.clone()} options={get_step_type_list()} onchange={clear_err_msg.reform(|_| ())} />
                                    </td>
                                </tr>
                                {
                                    match step_type.deref() {
                                        StepType::Auto => {
                                            let extension_list = extension_list.clone();
                                            let schema_resource_id = auto_step.schema_resource_id.clone();
                                            let operation_id = auto_step.operation_id.clone();
                                            let operation_parameter = auto_step.operation_parameter.clone();
                                            let environment_schema_detail = environment_schema_detail.clone();
                                            html! {
                                                schema_resource_id.data.view(move |schema_resource_id: UseStateHandle<Option<Id>>| {
                                                    let schema_resource_list: Vec<(Id,String)> = environment_schema_detail.as_ref().map(|detail| {
                                                        detail.resource_list.iter().map(|resource| {
                                                            (resource.id, resource.name.clone())
                                                        }).collect()
                                                    }).unwrap_or_default();
                                                    let extension_list = extension_list.clone();
                                                    let extension_list_clone = extension_list.clone();
                                                    let environment_schema_detail = environment_schema_detail.clone();
                                                    let environment_schema_detail_clone = environment_schema_detail.clone();
                                                    let operation_parameter = operation_parameter.clone();
                                                    let operation_parameter_clone = operation_parameter.clone();
                                                    let operation_parameter_clone2 = operation_parameter.clone();
                                                    let schema_resource_id_clone = schema_resource_id.clone();
                                                    let operation_id = operation_id.clone();
                                                    let operation_id_clone = operation_id.clone();
                                                    let on_schema_resource_change = Callback::from(move |schema_resource| {
                                                        if let Some((schema_resource_id, _)) = schema_resource {
                                                            let extension_id = get_extension_id(&environment_schema_detail_clone, schema_resource_id);
                                                            let operations: Vec<OperationOption> = extension_id.map(|extension_id| get_operations(
                                                                &extension_list_clone,
                                                                &extension_id,
                                                            ).map(|operations| {
                                                                operations.iter().map(|operation| {
                                                                    OperationOption(operation.clone())
                                                                }).collect()
                                                            })).flatten().unwrap_or_default();
                                                            if let Some(operation) = operations.first() {
                                                                if let Some(operation_id) = operation_id_clone.get_state() {
                                                                    operation_id.set(Some(operation.0.id.clone()));
                                                                }
                                                                if let Some(operation_parameter) = operation_parameter.get_state() {
                                                                    let default_parameter = get_default_config(operation.0.parameter_schema.clone());
                                                                    operation_parameter.set(default_parameter);
                                                                }
                                                            } else {
                                                                //把对应的操作清除掉
                                                                if let Some(operation_id) = operation_id_clone.get_state() {
                                                                    operation_id.set(None);
                                                                }
                                                                if let Some(operation_parameter) = operation_parameter.get_state() {
                                                                    operation_parameter.set(Default::default());
                                                                }
                                                            }
                                                        }
                                                    });
                                                    html! {
                                                        <>
                                                            <tr>
                                                                <td class="align-right"><Required/>{"操作资源："}</td>
                                                                <td>
                                                                    <SchemaResourceSelection value={schema_resource_id} options={schema_resource_list} onchange={on_schema_resource_change}/>
                                                                </td>
                                                            </tr>
                                                            <tr>
                                                                <td class="align-right" style="vertical-align: top;"><Required/>{"操作类型："}</td>
                                                                <td>
                                                                    {
                                                                        operation_id.view(move |operation_id, _validator| {
                                                                            let extension_list = extension_list.clone();
                                                                            let environment_schema_detail = environment_schema_detail.clone();
                                                                            let schema_resource_id = schema_resource_id_clone.clone();
                                                                            let operation_parameter = operation_parameter_clone.clone();
                                                                            let extension_id = schema_resource_id.as_ref().map(|schema_resource_id| {
                                                                                get_extension_id(
                                                                                    &environment_schema_detail,
                                                                                    *schema_resource_id,
                                                                                )
                                                                            }).flatten();
                                                                            let operations: Vec<OperationOption> = extension_id.map(|extension_id| get_operations(
                                                                                &extension_list,
                                                                                &extension_id,
                                                                            ).map(|operations| {
                                                                                operations.iter().map(|operation| {
                                                                                    OperationOption(operation.clone())
                                                                                }).collect()
                                                                            })).flatten().unwrap_or_default();
                                                                            let on_operation_change = Callback::from(move |operation: Option<OperationOption>| {
                                                                                if let (Some(operation), Some(operation_parameter)) = (operation, operation_parameter.get_state()) {
                                                                                    let extension_id = schema_resource_id.as_ref().map(|schema_resource_id| {
                                                                                        get_extension_id(
                                                                                            &environment_schema_detail,
                                                                                            *schema_resource_id,
                                                                                        )
                                                                                    }).flatten();
                                                                                    let default_parameter = extension_id.map(|extension_id| get_operations(
                                                                                        &extension_list,
                                                                                        &extension_id,
                                                                                    ).map(|operations| {
                                                                                        operations.iter().find(|item| {
                                                                                            item.id == operation.0.id
                                                                                        }).map(|operation| get_default_config(operation.parameter_schema.clone()))
                                                                                    })).flatten().flatten().unwrap_or_default();
                                                                                    operation_parameter.set(default_parameter);
                                                                                }
                                                                            });
                                                                            html! {
                                                                                <OperationSelection value={operation_id} options={operations.clone()} onchange={on_operation_change}/>
                                                                            }
                                                                        })
                                                                    }
                                                                </td>
                                                            </tr>
                                                            <tr>
                                                                <td class="align-right" style="vertical-align: top;">{"操作参数："}</td>
                                                                <td>
                                                                    {
                                                                        operation_parameter_clone2.view(move |operation_parameter: UseStateHandle<Vec<(Key, Attribute, AttributeValue)>>| {
                                                                            config_view(&operation_parameter)
                                                                        })
                                                                    }
                                                                </td>
                                                            </tr>
                                                        </>
                                                    }
                                                })
                                            }
                                        },
                                        StepType::Manual => html!{
                                            attachments.view(move |files: UseStateHandle<Vec<(Key, File, ())>>| {
                                                html! {
                                                    <tr>
                                                        <td class="align-right" style="vertical-align: top;">{"附件："}</td>
                                                        <td>
                                                            <BindingFilesUpload<()> files={files}/>
                                                        </td>
                                                    </tr>
                                                }
                                            })
                                        }
                                    }
                                }
                            </>
                        }
                    })
                }
            </table>
        }
    }
}

fn init_step_name(value: LightString) -> ValidateData<LightString> {
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
    extension_list: &UseStateHandle<Vec<Extension>>,
) -> Result<Vec<Extension>, LightString> {
    let result = QueryExtensionApi.call(&QueryExtensionReq {}).await?;
    extension_list.set(result.clone());
    return Ok(result);
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

async fn read_job_detail(
    edit_form: &EditForm,
    extension_list: &[Extension],
    environment_schema_detail: &UseStateHandle<Option<EnvironmentSchemaDetail>>,
    id: Id,
) -> Result<Job, LightString> {
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
                                step_type: Binding::new(StepType::Auto),
                                id: Binding::new(Some(*id)),
                                name: init_step_name(name.clone().into()),
                                remark: Binding::new(
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
                                    operation_parameter: Binding::new(operation_parameter),
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
                            step_type: Binding::new(StepType::Manual),
                            id: Binding::new(Some(*id)),
                            name: init_step_name(name.clone().into()),
                            remark: Binding::new(
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
                            attachments: Binding::new(
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
    environment_schema_detail: &UseStateHandle<Option<EnvironmentSchemaDetail>>,
    schema_resource_id: Id,
) -> Option<String> {
    return environment_schema_detail
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
    let mut active_step_key = None;
    for (step_key, job_step) in edit_form.job_step_list.iter() {
        if let Err(error) = job_step.name.validate(true) {
            err_msgs.push(error);
            if active_step_key.is_none() {
                active_step_key.replace(step_key.clone());
            }
        }
        let step_type = job_step.step_type.get();
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
    if let Some(active_step_key) = active_step_key {
        edit_form.active_job_step_key.set(Some(active_step_key));
        utils::wait(0).await;
        utils::trigger_resize();
    }
    return err_msgs;
}

async fn try_upload_files(edit_form: &EditForm) -> Result<(), LightString> {
    let mut files = Vec::new();
    for (_, job_step) in edit_form.job_step_list.deref().iter() {
        let remark = job_step.remark.get();
        upload_resource(&remark).await.map_err(|err| {
            log::error!("上传富文本资源失败: {:?}", err);
            return LightString::from("上传文件失败");
        })?;
        let step_type = job_step.step_type.get();
        match step_type {
            StepType::Auto => {
                let operation_parameter = job_step.auto_step.operation_parameter.get();
                for (_, _, attr_value) in operation_parameter {
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
                            for (index, (_key, file, ())) in file_list.get().into_iter().enumerate()
                            {
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
            StepType::Manual => {
                let new_files = job_step.attachments.get();
                let file_count = new_files.len();
                let finished_count = 0;
                let lock_data = Arc::new(Mutex::new((new_files, finished_count)));
                for (index, (_key, file, ())) in job_step.attachments.get().into_iter().enumerate()
                {
                    let lock_data = lock_data.clone();
                    match file {
                        File::Local(hashing_file) => {
                            let attachments = job_step.attachments.clone();
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
                                        attachments.set(new_files.clone());
                                    }
                                }),
                            ));
                        }
                        File::Remote { .. } => (),
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
    let mut job_step_list: Vec<_> = Vec::with_capacity(edit_form.job_step_list.len());
    for (_, job_step) in edit_form.job_step_list.deref().iter() {
        let step_type = job_step.step_type.get();
        let id = job_step.id.get();
        let name = job_step.name.get();
        let remark = job_step.remark.get();
        let remark = Some(JSON::stringify(&remark).unwrap().as_string().unwrap());
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
    return job_step_list;
}

async fn save_job(
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
    let job_step_list = collect_job_step_list_list(edit_form);
    let name = edit_form.name.get().to_string();
    let remark = if edit_form.remark.is_empty() {
        None
    } else {
        Some(edit_form.remark.to_string())
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
                        onsave.emit(tihu::PrimaryKey { id: id });
                    }
                    None => (),
                }
                utils::success(LightString::from("保存成功"));
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
