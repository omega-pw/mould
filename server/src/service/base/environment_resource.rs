use tihu_native::errno::execute_error;
use tihu_native::errno::extract_data_error;
use tihu_native::errno::prepare_statement_error;
use tihu_native::errno::query_error;
use tihu_native::errno::undefined_enum_value;
use tihu_native::ErrNo;
use tihu::Id;
use tihu::LightString;
use lazy_static;
use format_xml;
use std::borrow::Cow;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Row, Transaction};
use native_common::utils::add_val;
use native_common::utils::add_vals;
use native_common::utils::Condition;
use native_common::utils::add_conditions;
use native_common::utils::calc_sql_pagination;
use crate::log;
use crate::model::environment_resource::properties;
use crate::model::environment_resource::EnvironmentResource;
use crate::model::environment_resource::EnvironmentResourceProperty;
use crate::model::environment_resource::EnvironmentResourceOpt;
use crate::native_common;

const ENTITY: &str = "environment_resource";
const EXTRA_PROPERTIES: [&str; 9] = [properties::ORG_ID,properties::ENVIRONMENT_ID,properties::SCHEMA_RESOURCE_ID,properties::NAME,properties::EXTENSION_ID,properties::EXTENSION_NAME,properties::EXTENSION_CONFIGURATION,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_environment_resource(row: &Row) -> Result<EnvironmentResource, ErrNo> {
    return Ok(EnvironmentResource {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row.try_get(properties::ORG_ID).map_err(extract_data_error)?,
        environment_id: row.try_get(properties::ENVIRONMENT_ID).map_err(extract_data_error)?,
        schema_resource_id: row.try_get(properties::SCHEMA_RESOURCE_ID).map_err(extract_data_error)?,
        name: row.try_get(properties::NAME).map_err(extract_data_error)?,
        extension_id: row.try_get(properties::EXTENSION_ID).map_err(extract_data_error)?,
        extension_name: row.try_get(properties::EXTENSION_NAME).map_err(extract_data_error)?,
        extension_configuration: row.try_get(properties::EXTENSION_CONFIGURATION).map_err(extract_data_error)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a EnvironmentResourceOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
    let mut pairs = Vec::<(Condition,&(dyn ToSql + std::marker::Sync))>::new();
    if let Some(id) = opt.id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ID), operator: None}, id));
    }
    if let Some(org_id) = opt.org_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ORG_ID), operator: None}, org_id));
    }
    if let Some(environment_id) = opt.environment_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ENVIRONMENT_ID), operator: None}, environment_id));
    }
    if let Some(schema_resource_id) = opt.schema_resource_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::SCHEMA_RESOURCE_ID), operator: None}, schema_resource_id));
    }
    if let Some(name) = opt.name.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::NAME), operator: None}, name));
    }
    if let Some(extension_id) = opt.extension_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::EXTENSION_ID), operator: None}, extension_id));
    }
    if let Some(extension_name) = opt.extension_name.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::EXTENSION_NAME), operator: None}, extension_name));
    }
    if let Some(extension_configuration) = opt.extension_configuration.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::EXTENSION_CONFIGURATION), operator: None}, extension_configuration));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::CREATED_TIME), operator: None}, created_time));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::LAST_MODIFIED_TIME), operator: None}, last_modified_time));
    }
    return pairs;
}

pub struct EnvironmentResourceBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> EnvironmentResourceBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> EnvironmentResourceBaseService<'a> {
        return EnvironmentResourceBaseService {
            transaction: transaction
        };
    }

    pub async fn read_environment_resource(&self, id: Id) -> Result<Option<EnvironmentResource>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_environment_resource).transpose()?);
    }

    pub async fn read_environment_resource_batch(&self, ids: &[Id]) -> Result<Vec<EnvironmentResource>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的环境资源id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<EnvironmentResource> = rows.iter().map(extract_environment_resource).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_environment_resource(&self, environment_resource: &EnvironmentResource) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &environment_resource.id),
                    add_val(&mut vals, &environment_resource.org_id),
                    add_val(&mut vals, &environment_resource.environment_id),
                    add_val(&mut vals, &environment_resource.schema_resource_id),
                    add_val(&mut vals, &environment_resource.name),
                    add_val(&mut vals, &environment_resource.extension_id),
                    add_val(&mut vals, &environment_resource.extension_name),
                    add_val(&mut vals, &environment_resource.extension_configuration),
                    add_val(&mut vals, &environment_resource.created_time),
                    add_val(&mut vals, &environment_resource.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_environment_resource_batch(&self, environment_resource_list: &[EnvironmentResource]) -> Result<(), ErrNo> {
        if environment_resource_list.is_empty() {
            log::warn!("待批量新增的环境资源集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * environment_resource_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                environment_resource_list.iter().map(|environment_resource|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &environment_resource.id),
                        add_val(&mut vals, &environment_resource.org_id),
                        add_val(&mut vals, &environment_resource.environment_id),
                        add_val(&mut vals, &environment_resource.schema_resource_id),
                        add_val(&mut vals, &environment_resource.name),
                        add_val(&mut vals, &environment_resource.extension_id),
                        add_val(&mut vals, &environment_resource.extension_name),
                        add_val(&mut vals, &environment_resource.extension_configuration),
                        add_val(&mut vals, &environment_resource.created_time),
                        add_val(&mut vals, &environment_resource.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_environment_resource_full(&self, environment_resource: &EnvironmentResource) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::ORG_ID, "=", &add_val(&mut vals, &environment_resource.org_id),
                    ",", properties::ENVIRONMENT_ID, "=", &add_val(&mut vals, &environment_resource.environment_id),
                    ",", properties::SCHEMA_RESOURCE_ID, "=", &add_val(&mut vals, &environment_resource.schema_resource_id),
                    ",", properties::NAME, "=", &add_val(&mut vals, &environment_resource.name),
                    ",", properties::EXTENSION_ID, "=", &add_val(&mut vals, &environment_resource.extension_id),
                    ",", properties::EXTENSION_NAME, "=", &add_val(&mut vals, &environment_resource.extension_name),
                    ",", properties::EXTENSION_CONFIGURATION, "=", &add_val(&mut vals, &environment_resource.extension_configuration),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &environment_resource.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &environment_resource.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &environment_resource.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_environment_resource(&self, id: Id, changes: &[EnvironmentResourceProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&EnvironmentResourceProperty> = changes
            .iter()
            .filter(|change| match change {
                EnvironmentResourceProperty::Id(_) => false,
                _ => true,
            })
            .collect();
        if changes.is_empty() {
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1 + changes.len());
        let mut trunks: Vec<Cow<str>> = Vec::with_capacity(4 * changes.len());
        for (index, property) in changes.iter().enumerate() {
            if 0 < index {
                trunks.push(",".into());
            }
            trunks.extend(match property {
                EnvironmentResourceProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                EnvironmentResourceProperty::OrgId(org_id) => {
                    [properties::ORG_ID.into(), "=".into(), add_val(&mut vals, org_id).into()]
                },
                EnvironmentResourceProperty::EnvironmentId(environment_id) => {
                    [properties::ENVIRONMENT_ID.into(), "=".into(), add_val(&mut vals, environment_id).into()]
                },
                EnvironmentResourceProperty::SchemaResourceId(schema_resource_id) => {
                    [properties::SCHEMA_RESOURCE_ID.into(), "=".into(), add_val(&mut vals, schema_resource_id).into()]
                },
                EnvironmentResourceProperty::Name(name) => {
                    [properties::NAME.into(), "=".into(), add_val(&mut vals, name).into()]
                },
                EnvironmentResourceProperty::ExtensionId(extension_id) => {
                    [properties::EXTENSION_ID.into(), "=".into(), add_val(&mut vals, extension_id).into()]
                },
                EnvironmentResourceProperty::ExtensionName(extension_name) => {
                    [properties::EXTENSION_NAME.into(), "=".into(), add_val(&mut vals, extension_name).into()]
                },
                EnvironmentResourceProperty::ExtensionConfiguration(extension_configuration) => {
                    [properties::EXTENSION_CONFIGURATION.into(), "=".into(), add_val(&mut vals, extension_configuration).into()]
                },
                EnvironmentResourceProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                EnvironmentResourceProperty::LastModifiedTime(last_modified_time) => {
                    [properties::LAST_MODIFIED_TIME.into(), "=".into(), add_val(&mut vals, last_modified_time).into()]
                },
            });
        }
        let change_content: String = trunks.concat();
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {change_content} " where "{properties::ID}"=" {add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_environment_resource(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_environment_resource_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的环境资源id集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn query_environment_resource_count(&self, opt: &EnvironmentResourceOpt) -> Result<u64, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select count(1) from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row: Row = self.transaction.query_one(&statement, &vals).await.map_err(query_error)?;
        let count:i64 = row.get(0);
        return Ok(count as u64);
    }

    pub async fn query_environment_resource(&self, page_no: u64, page_size: u64, opt: &EnvironmentResourceOpt) -> Result<Vec<EnvironmentResource>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let (limit, offset) = calc_sql_pagination(page_no, page_size);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
            " limit "{limit}" offset "{offset}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<EnvironmentResource> = rows.iter().map(extract_environment_resource).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_environment_resource_one(&self, opt: &EnvironmentResourceOpt) -> Result<Option<EnvironmentResource>, ErrNo> {
        let list = self.query_environment_resource(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_environment_resource_batch(&self, opt: &EnvironmentResourceOpt) -> Result<Vec<EnvironmentResource>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<EnvironmentResource> = rows.iter().map(extract_environment_resource).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}