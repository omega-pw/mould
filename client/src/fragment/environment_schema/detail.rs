use crate::components::table::ArcRowRenderer;
use crate::components::table::Column;
use crate::components::table::Table;
use crate::components::ArcRenderer;
use crate::sdk;
use crate::utils;
use crate::utils::request::ApiExt;
use crate::LightString;
use sdk::environment_schema::read_environment_schema::EnvironmentSchema;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaApi;
use sdk::environment_schema::read_environment_schema::ReadEnvironmentSchemaReq;
use sdk::environment_schema::read_environment_schema::SchemaResource;
use tihu::Id;
use yew::prelude::*;
use yew::virtual_dom::Key;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub id: Id,
}

#[function_component]
pub fn EnvironmentSchemaDetail(props: &Props) -> Html {
    let detail: UseStateHandle<Option<EnvironmentSchema>> = use_state(|| None);
    let id = props.id;
    let detail_clone = detail.clone();
    use_effect_with(id, move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            read_environment_schema_detail(&detail_clone, id).await.ok();
        });
        || ()
    });
    let columns: Vec<Column<SchemaResource>> = vec![
        Column {
            key: "name".into(),
            head: ArcRenderer::from(move |_: &'_ ()| {
                html! { "规格名称" }
            }),
            row: ArcRowRenderer::from(move |schema_resource: &SchemaResource, _index: usize| {
                Html::from(&schema_resource.name)
            }),
            head_style: None,
            data_style: None,
        },
        Column {
            key: "extension".into(),
            head: ArcRenderer::from(move |_: &'_ ()| {
                html! { "资源类型" }
            }),
            row: ArcRowRenderer::from(move |schema_resource: &SchemaResource, _index: usize| {
                Html::from(&schema_resource.extension_name)
            }),
            head_style: None,
            data_style: None,
        },
    ];
    html! {
        <div class="width-fill height-fill border-box" style="padding:0.25em;">
            <table class="width-fill" style="border-collapse:collapse;table-layout: fixed;">
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"环境规格名称："}</td>
                    <td>{detail.as_ref().map(|environment_schema|{html!{&environment_schema.name}}).unwrap_or_else(utils::empty_html)}</td>
                </tr>
                <tr>
                    <td class="align-right" style="width:8em;vertical-align: top;">{"资源规格："}</td>
                    <td>
                        {
                            if let Some(detail) = detail.as_ref() {
                                let resource_list: Vec<(Key, SchemaResource)> = detail.resource_list.iter().map(|resource| {
                                    (resource.id.to_string().into(), resource.clone())
                                }).collect();
                                html! {
                                    <Table<SchemaResource> list={resource_list} columns={columns} />
                                }
                            } else {
                                html! {}
                            }
                        }
                    </td>
                </tr>
            </table>
        </div>
    }
}

async fn read_environment_schema_detail(
    detail: &UseStateHandle<Option<EnvironmentSchema>>,
    id: Id,
) -> Result<(), LightString> {
    let params = ReadEnvironmentSchemaReq { id: id };
    let environment_schema = ReadEnvironmentSchemaApi.call(&params).await?;
    detail.set(Some(environment_schema));
    return Ok(());
}
