use crate::components::table::ArcRowRenderer;
use crate::components::table::Column;
use crate::components::table::Table;
use crate::components::ArcRenderer;
use crate::sdk;
use crate::utils::request::ApiExt;
use crate::SharedString;
use leptos::prelude::*;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::environment_schema::read_environment_schema::SchemaResource;
use tihu::Id;

#[component]
pub fn EnvironmentSchemaDetail(#[prop(optional)] id: Id) -> impl IntoView {
    let detail: RwSignal<Option<EnvironmentSchema>> = RwSignal::new(None);
    let detail_clone = detail.clone();
    wasm_bindgen_futures::spawn_local(async move {
        read_environment_schema_detail(&detail_clone, id).await.ok();
    });
    let columns: Vec<Column<SchemaResource>> = vec![
        Column {
            key: "name".into(),
            head: ArcRenderer::from(move |_: &'_ ()| "规格名称".into_any()),
            row: ArcRowRenderer::from(move |schema_resource: &SchemaResource, _index: usize| {
                schema_resource.name.clone().into_any()
            }),
            head_style: None,
            data_style: None,
        },
        Column {
            key: "extension".into(),
            head: ArcRenderer::from(move |_: &'_ ()| "资源类型".into_any()),
            row: ArcRowRenderer::from(move |schema_resource: &SchemaResource, _index: usize| {
                schema_resource.extension_name.clone().into_any()
            }),
            head_style: None,
            data_style: None,
        },
    ];
    view! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"环境规格名称："}</td>
                    <td>
                        {
                            let detail = detail.clone();
                            move || detail.read().as_ref().map(|environment_schema| environment_schema.name.clone())
                        }
                    </td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"资源规格："}</td>
                    <td>
                        <Table list={Signal::derive(move || {
                            detail.read().as_ref().map(|detail|detail.resource_list.iter().map(|resource| {
                                (SharedString::from(resource.id.to_string()), resource.clone())
                            }).collect::<Vec<_>>()).unwrap_or_default()
                        })} columns={columns} />
                    </td>
                </tr>
            </table>
        </div>
    }
}

async fn read_environment_schema_detail(
    detail: &RwSignal<Option<EnvironmentSchema>>,
    id: Id,
) -> Result<(), SharedString> {
    let params = ReadEnvironmentSchemaReq { id: id };
    let environment_schema = ReadEnvironmentSchemaApi.call(&params).await?;
    detail.set(Some(environment_schema));
    return Ok(());
}
