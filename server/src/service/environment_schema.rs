use crate::log;
use crate::model::environment_schema::properties;
use crate::model::environment_schema::EnvironmentSchema;
use crate::model::environment_schema::EnvironmentSchemaOpt;
use crate::model::environment_schema::EnvironmentSchemaProperty;
use crate::native_common;
use format_xml;
use lazy_static;
use native_common::utils::add_conditions;
use native_common::utils::add_val;
use native_common::utils::add_vals;
use native_common::utils::calc_sql_pagination;
use native_common::utils::Condition;
use std::borrow::Cow;
use tihu::LightString;
use tihu_native::errno::execute_error;
use tihu_native::errno::extract_data_error;
use tihu_native::errno::prepare_statement_error;
use tihu_native::errno::query_error;
use tihu_native::errno::undefined_enum_value;
use tihu_native::ErrNo;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Row, Transaction};

const ENTITY: &str = "environment_schema";
const EXTRA_PROPERTIES: [&str; 4] = [
    properties::ORG_ID,
    properties::NAME,
    properties::CREATED_TIME,
    properties::LAST_MODIFIED_TIME,
];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len() + 1;

fn gen_properties() -> String {
    let properties: Vec<&str> = [properties::ID]
        .iter()
        .chain(EXTRA_PROPERTIES.iter())
        .map(|item| *item)
        .collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_environment_schema(row: &Row) -> Result<EnvironmentSchema, ErrNo> {
    return Ok(EnvironmentSchema {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row
            .try_get(properties::ORG_ID)
            .map_err(extract_data_error)?,
        name: row.try_get(properties::NAME).map_err(extract_data_error)?,
        created_time: row
            .try_get(properties::CREATED_TIME)
            .map_err(extract_data_error)?,
        last_modified_time: row
            .try_get(properties::LAST_MODIFIED_TIME)
            .map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(
    opt: &'a EnvironmentSchemaOpt,
) -> Vec<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
    let mut pairs = Vec::<(Condition, &(dyn ToSql + std::marker::Sync))>::new();
    if let Some(id) = opt.id.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::ID),
                operator: None,
            },
            id,
        ));
    }
    if let Some(name) = opt.name.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::NAME),
                operator: None,
            },
            name,
        ));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::CREATED_TIME),
                operator: None,
            },
            created_time,
        ));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::LAST_MODIFIED_TIME),
                operator: None,
            },
            last_modified_time,
        ));
    }
    return pairs;
}

pub struct EnvironmentSchemaService<'a> {
    transaction: &'a Transaction<'a>,
}

impl<'a> EnvironmentSchemaService<'a> {
    pub fn new(transaction: &'a Transaction) -> EnvironmentSchemaService<'a> {
        return EnvironmentSchemaService {
            transaction: transaction,
        };
    }

    pub async fn query_environment_schema(
        &self,
        page_no: u64,
        page_size: u64,
        opt: &EnvironmentSchemaOpt,
    ) -> Result<Vec<EnvironmentSchema>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let (limit, offset) = calc_sql_pagination(page_no, page_size);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
            " order by "{properties::CREATED_TIME}" desc limit "{limit}" offset "{offset}
        }
        .to_string();
        let statement = self
            .transaction
            .prepare(&sql)
            .await
            .map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self
            .transaction
            .query(&statement, &vals)
            .await
            .map_err(query_error)?;
        let list: Vec<EnvironmentSchema> = rows
            .iter()
            .map(extract_environment_schema)
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }
}
