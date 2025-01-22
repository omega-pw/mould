use super::super::job_record::detail::JobRecordDetail;
use crate::components::button::Button;
use crate::components::r#if::If;
use crate::components::selection::BindingSelection;
use crate::sdk;
use crate::utils::request::ApiExt;
use crate::LightString;
use sdk::environment::query_environment::Environment;
use sdk::environment::query_environment::QueryEnvironmentApi;
use sdk::environment::query_environment::QueryEnvironmentReq;
use sdk::job::read_job::Job;
use sdk::job::read_job::ReadJobApi;
use sdk::job::read_job::ReadJobReq;
use sdk::job::start_job::StartJobApi;
use sdk::job::start_job::StartJobReq;
use tihu::Id;
use tihu::PrimaryKey;
use yew::prelude::*;

type EnvironmentSelection = BindingSelection<(Id, String)>;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Id,
}

#[function_component]
pub fn StartJob(props: &Props) -> Html {
    let record_detail_active: UseStateHandle<bool> = use_state(|| false);
    let active_record_detail_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let detail: UseStateHandle<Option<Job>> = use_state(|| None);
    let active_environment_id: UseStateHandle<Option<Id>> = use_state(|| None);
    let environment_list: UseStateHandle<Vec<Environment>> = use_state(|| Vec::new());
    let id = props.id;
    let detail_clone = detail.clone();
    let environment_list_clone = environment_list.clone();
    let active_environment_id_clone = active_environment_id.clone();
    use_effect_with(id, move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            match read_job_detail(&detail_clone, id).await {
                Ok(job) => {
                    let environment_schema_id = job.environment_schema_id;
                    match query_environment_list(environment_schema_id, &environment_list_clone)
                        .await
                    {
                        Ok(job) => {
                            if let Some(first) = job.first() {
                                active_environment_id_clone.set(Some(first.id));
                            }
                        }
                        Err(_err) => {
                            //
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
    let environment_list: Vec<_> = environment_list
        .iter()
        .map(|environment| (environment.id, environment.name.clone()))
        .collect();
    let job_id = id;
    let active_environment_id_clone = active_environment_id.clone();
    let record_detail_active_clone = record_detail_active.clone();
    let active_record_detail_id_clone = active_record_detail_id.clone();
    let on_run = Callback::from(move |_: ()| {
        if let Some(active_environment_id) = active_environment_id_clone.as_ref() {
            let active_environment_id = *active_environment_id;
            let record_detail_active = record_detail_active_clone.clone();
            let active_record_detail_id = active_record_detail_id_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match start_job(job_id, active_environment_id).await {
                    Ok(job_record) => {
                        active_record_detail_id.set(Some(job_record.id));
                        record_detail_active.set(true);
                    }
                    Err(_err) => {
                        //
                    }
                }
            });
        }
    });
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <If condition={!*record_detail_active}>
                {"环境: "}
                <EnvironmentSelection value={active_environment_id.clone()} options={environment_list}/>
                <Button disabled={active_environment_id.is_none()} onclick={on_run} style="margin-left:0.5em;">{"执行"}</Button>
            </If>
            {
                if let (true, Some(active_record_detail_id)) = (*record_detail_active, active_record_detail_id.as_ref()) {
                    html! {
                        <JobRecordDetail id={*active_record_detail_id} />
                    }
                } else {
                    html!{}
                }
            }
        </div>
    }
}

async fn read_job_detail(detail: &UseStateHandle<Option<Job>>, id: Id) -> Result<Job, LightString> {
    let params = ReadJobReq { id: id };
    let job = ReadJobApi.call(&params).await?;
    detail.set(Some(job.clone()));
    return Ok(job);
}

async fn query_environment_list(
    environment_schema_id: Id,
    environment_list: &UseStateHandle<Vec<Environment>>,
) -> Result<Vec<Environment>, LightString> {
    let pagination_list = QueryEnvironmentApi
        .call(&QueryEnvironmentReq {
            environment_schema_id: Some(environment_schema_id),
            page_no: Some(1),
            ..QueryEnvironmentReq::empty()
        })
        .await?;
    environment_list.set(pagination_list.list.clone());
    return Ok(pagination_list.list);
}

async fn start_job(job_id: Id, environment_id: Id) -> Result<PrimaryKey, LightString> {
    let job_record = StartJobApi
        .call(&StartJobReq {
            job_id: job_id,
            environment_id: environment_id,
        })
        .await?;
    return Ok(job_record);
}
