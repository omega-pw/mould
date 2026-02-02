use super::super::job_record::detail::JobRecordDetail;
use crate::components::button::Button;
use crate::components::selection::Selection;
use crate::sdk;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
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

#[component]
pub fn StartJob(#[prop(optional)] id: Id) -> impl IntoView {
    let record_detail_active: RwSignal<bool> = RwSignal::new(false);
    let active_record_detail_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let detail: RwSignal<Option<Job>> = RwSignal::new(None);
    let active_environment_id: RwSignal<Option<Id>> = RwSignal::new(None);
    let environment_list: RwSignal<Vec<Environment>> = RwSignal::new(Vec::new());
    let detail_clone = detail.clone();
    let environment_list_clone = environment_list.clone();
    let active_environment_id_clone = active_environment_id.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match read_job_detail(&detail_clone, id).await {
            Ok(job) => {
                let environment_schema_id = job.environment_schema_id;
                match query_environment_list(environment_schema_id, &environment_list_clone).await {
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
    let environment_list = Signal::derive(move || {
        environment_list
            .read()
            .iter()
            .map(|environment| (environment.id, environment.name.clone()))
            .collect()
    });
    let job_id = id;
    let on_run = {
        let active_environment_id: RwSignal<Option<uuid::Uuid>> = active_environment_id.clone();
        let record_detail_active = record_detail_active.clone();
        let active_record_detail_id = active_record_detail_id.clone();
        UnsyncCallback::new(move |_: ()| {
            if let Some(active_environment_id) = active_environment_id.get() {
                let active_record_detail_id = active_record_detail_id.clone();
                let record_detail_active = record_detail_active.clone();
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
        })
    };
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <Show when={
                let record_detail_active = record_detail_active.clone();
                move || record_detail_active.get()
            }>
                {"环境: "}
                <Selection value={active_environment_id.clone()} options={environment_list}/>
                <Button disabled={move || active_environment_id.read().is_none()} onclick={on_run} style={SharedString::from("margin-left:0.5em;")}>{"执行"}</Button>
            </Show>
            {
                move || {
                    if let (true, Some(active_record_detail_id)) = (record_detail_active.get(), active_record_detail_id.get()) {
                        view! {
                            <JobRecordDetail id={active_record_detail_id} />
                        }.into_any()
                    } else {
                        view!{}.into_any()
                    }
                }
            }
        </div>
    }
}

async fn read_job_detail(detail: &RwSignal<Option<Job>>, id: Id) -> Result<Job, SharedString> {
    let params = ReadJobReq { id: id };
    let job = ReadJobApi.call(&params).await?;
    detail.set(Some(job.clone()));
    return Ok(job);
}

async fn query_environment_list(
    environment_schema_id: Id,
    environment_list: &RwSignal<Vec<Environment>>,
) -> Result<Vec<Environment>, SharedString> {
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

async fn start_job(job_id: Id, environment_id: Id) -> Result<PrimaryKey, SharedString> {
    let job_record = StartJobApi
        .call(&StartJobReq {
            job_id: job_id,
            environment_id: environment_id,
        })
        .await?;
    return Ok(job_record);
}
