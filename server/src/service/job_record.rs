use crate::log;
use crate::model::job_record::enums::try_i16_to_status;
use crate::model::job_record::properties;
use crate::model::job_record::JobRecord;
use crate::model::job_record::JobRecordOpt;
use crate::model::job_record::JobRecordProperty;
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

const ENTITY: &str = "job_record";
const EXTRA_PROPERTIES: [&str; 6] = [
    properties::ORG_ID,
    properties::JOB_ID,
    properties::ENVIRONMENT_ID,
    properties::STATUS,
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

fn extract_job_record(row: &Row) -> Result<JobRecord, ErrNo> {
    return Ok(JobRecord {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row
            .try_get(properties::ORG_ID)
            .map_err(extract_data_error)?,
        job_id: row
            .try_get(properties::JOB_ID)
            .map_err(extract_data_error)?,
        environment_id: row
            .try_get(properties::ENVIRONMENT_ID)
            .map_err(extract_data_error)?,
        status: try_i16_to_status(
            row.try_get(properties::STATUS)
                .map_err(extract_data_error)?,
        )
        .map_err(undefined_enum_value)?,
        created_time: row
            .try_get(properties::CREATED_TIME)
            .map_err(extract_data_error)?,
        last_modified_time: row
            .try_get(properties::LAST_MODIFIED_TIME)
            .map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(
    opt: &'a JobRecordOpt,
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
    if let Some(job_id) = opt.job_id.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::JOB_ID),
                operator: None,
            },
            job_id,
        ));
    }
    if let Some(environment_id) = opt.environment_id.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::ENVIRONMENT_ID),
                operator: None,
            },
            environment_id,
        ));
    }
    if let Some(status) = opt.status.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::STATUS),
                operator: None,
            },
            status,
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

pub struct JobRecordService<'a> {
    transaction: &'a Transaction<'a>,
}

impl<'a> JobRecordService<'a> {
    pub fn new(transaction: &'a Transaction) -> JobRecordService<'a> {
        return JobRecordService {
            transaction: transaction,
        };
    }

    pub async fn query_job_record(
        &self,
        page_no: u64,
        page_size: u64,
        opt: &JobRecordOpt,
    ) -> Result<Vec<JobRecord>, ErrNo> {
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
        let list: Vec<JobRecord> = rows
            .iter()
            .map(extract_job_record)
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }
}
