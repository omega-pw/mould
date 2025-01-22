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
use crate::model::job_record::properties;
use crate::model::job_record::JobRecord;
use crate::model::job_record::JobRecordProperty;
use crate::model::job_record::JobRecordOpt;
use crate::model::job_record::enums::try_i16_to_status;
use crate::native_common;

const ENTITY: &str = "job_record";
const EXTRA_PROPERTIES: [&str; 6] = [properties::ORG_ID,properties::JOB_ID,properties::ENVIRONMENT_ID,properties::STATUS,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_job_record(row: &Row) -> Result<JobRecord, ErrNo> {
    return Ok(JobRecord {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row.try_get(properties::ORG_ID).map_err(extract_data_error)?,
        job_id: row.try_get(properties::JOB_ID).map_err(extract_data_error)?,
        environment_id: row.try_get(properties::ENVIRONMENT_ID).map_err(extract_data_error)?,
        status: try_i16_to_status(row.try_get(properties::STATUS).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a JobRecordOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
    let mut pairs = Vec::<(Condition,&(dyn ToSql + std::marker::Sync))>::new();
    if let Some(id) = opt.id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ID), operator: None}, id));
    }
    if let Some(org_id) = opt.org_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ORG_ID), operator: None}, org_id));
    }
    if let Some(job_id) = opt.job_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::JOB_ID), operator: None}, job_id));
    }
    if let Some(environment_id) = opt.environment_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ENVIRONMENT_ID), operator: None}, environment_id));
    }
    if let Some(status) = opt.status.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::STATUS), operator: None}, status));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::CREATED_TIME), operator: None}, created_time));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::LAST_MODIFIED_TIME), operator: None}, last_modified_time));
    }
    return pairs;
}

pub struct JobRecordBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> JobRecordBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> JobRecordBaseService<'a> {
        return JobRecordBaseService {
            transaction: transaction
        };
    }

    pub async fn read_job_record(&self, id: Id) -> Result<Option<JobRecord>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_job_record).transpose()?);
    }

    pub async fn read_job_record_batch(&self, ids: &[Id]) -> Result<Vec<JobRecord>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的任务记录id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobRecord> = rows.iter().map(extract_job_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_job_record(&self, job_record: &JobRecord) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &job_record.id),
                    add_val(&mut vals, &job_record.org_id),
                    add_val(&mut vals, &job_record.job_id),
                    add_val(&mut vals, &job_record.environment_id),
                    add_val(&mut vals, &job_record.status),
                    add_val(&mut vals, &job_record.created_time),
                    add_val(&mut vals, &job_record.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_job_record_batch(&self, job_record_list: &[JobRecord]) -> Result<(), ErrNo> {
        if job_record_list.is_empty() {
            log::warn!("待批量新增的任务记录集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * job_record_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                job_record_list.iter().map(|job_record|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &job_record.id),
                        add_val(&mut vals, &job_record.org_id),
                        add_val(&mut vals, &job_record.job_id),
                        add_val(&mut vals, &job_record.environment_id),
                        add_val(&mut vals, &job_record.status),
                        add_val(&mut vals, &job_record.created_time),
                        add_val(&mut vals, &job_record.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_record_full(&self, job_record: &JobRecord) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::ORG_ID, "=", &add_val(&mut vals, &job_record.org_id),
                    ",", properties::JOB_ID, "=", &add_val(&mut vals, &job_record.job_id),
                    ",", properties::ENVIRONMENT_ID, "=", &add_val(&mut vals, &job_record.environment_id),
                    ",", properties::STATUS, "=", &add_val(&mut vals, &job_record.status),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &job_record.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &job_record.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &job_record.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_record(&self, id: Id, changes: &[JobRecordProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&JobRecordProperty> = changes
            .iter()
            .filter(|change| match change {
                JobRecordProperty::Id(_) => false,
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
                JobRecordProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                JobRecordProperty::OrgId(org_id) => {
                    [properties::ORG_ID.into(), "=".into(), add_val(&mut vals, org_id).into()]
                },
                JobRecordProperty::JobId(job_id) => {
                    [properties::JOB_ID.into(), "=".into(), add_val(&mut vals, job_id).into()]
                },
                JobRecordProperty::EnvironmentId(environment_id) => {
                    [properties::ENVIRONMENT_ID.into(), "=".into(), add_val(&mut vals, environment_id).into()]
                },
                JobRecordProperty::Status(status) => {
                    [properties::STATUS.into(), "=".into(), add_val(&mut vals, status).into()]
                },
                JobRecordProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                JobRecordProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_job_record(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_job_record_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的任务记录id集合为空");
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

    pub async fn query_job_record_count(&self, opt: &JobRecordOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_job_record(&self, page_no: u64, page_size: u64, opt: &JobRecordOpt) -> Result<Vec<JobRecord>, ErrNo> {
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
        let list: Vec<JobRecord> = rows.iter().map(extract_job_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_job_record_one(&self, opt: &JobRecordOpt) -> Result<Option<JobRecord>, ErrNo> {
        let list = self.query_job_record(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_job_record_batch(&self, opt: &JobRecordOpt) -> Result<Vec<JobRecord>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobRecord> = rows.iter().map(extract_job_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}