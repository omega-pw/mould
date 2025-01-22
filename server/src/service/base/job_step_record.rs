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
use crate::model::job_step_record::properties;
use crate::model::job_step_record::JobStepRecord;
use crate::model::job_step_record::JobStepRecordProperty;
use crate::model::job_step_record::JobStepRecordOpt;
use crate::model::job_step_record::enums::try_i16_to_step_type;
use crate::model::job_step_record::enums::try_i16_to_status;
use crate::native_common;

const ENTITY: &str = "job_step_record";
const EXTRA_PROPERTIES: [&str; 17] = [properties::ORG_ID,properties::JOB_ID,properties::ENVIRONMENT_ID,properties::RECORD_ID,properties::JOB_STEP_ID,properties::STEP_NAME,properties::STEP_TYPE,properties::STEP_REMARK,properties::EXTENSION_ID,properties::OPERATION_ID,properties::OPERATION_NAME,properties::OPERATION_PARAMETER,properties::ATTACHMENTS,properties::JOB_STEP_SEQ,properties::STATUS,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_job_step_record(row: &Row) -> Result<JobStepRecord, ErrNo> {
    return Ok(JobStepRecord {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row.try_get(properties::ORG_ID).map_err(extract_data_error)?,
        job_id: row.try_get(properties::JOB_ID).map_err(extract_data_error)?,
        environment_id: row.try_get(properties::ENVIRONMENT_ID).map_err(extract_data_error)?,
        record_id: row.try_get(properties::RECORD_ID).map_err(extract_data_error)?,
        job_step_id: row.try_get(properties::JOB_STEP_ID).map_err(extract_data_error)?,
        step_name: row.try_get(properties::STEP_NAME).map_err(extract_data_error)?,
        step_type: try_i16_to_step_type(row.try_get(properties::STEP_TYPE).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        step_remark: row.try_get(properties::STEP_REMARK).map_err(extract_data_error)?,
        extension_id: row.try_get(properties::EXTENSION_ID).map_err(extract_data_error)?,
        operation_id: row.try_get(properties::OPERATION_ID).map_err(extract_data_error)?,
        operation_name: row.try_get(properties::OPERATION_NAME).map_err(extract_data_error)?,
        operation_parameter: row.try_get(properties::OPERATION_PARAMETER).map_err(extract_data_error)?,
        attachments: row.try_get(properties::ATTACHMENTS).map_err(extract_data_error)?,
        job_step_seq: row.try_get(properties::JOB_STEP_SEQ).map_err(extract_data_error)?,
        status: try_i16_to_status(row.try_get(properties::STATUS).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a JobStepRecordOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
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
    if let Some(record_id) = opt.record_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::RECORD_ID), operator: None}, record_id));
    }
    if let Some(job_step_id) = opt.job_step_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::JOB_STEP_ID), operator: None}, job_step_id));
    }
    if let Some(step_name) = opt.step_name.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::STEP_NAME), operator: None}, step_name));
    }
    if let Some(step_type) = opt.step_type.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::STEP_TYPE), operator: None}, step_type));
    }
    if let Some(step_remark) = opt.step_remark.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::STEP_REMARK), operator: None}, step_remark));
    }
    if let Some(extension_id) = opt.extension_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::EXTENSION_ID), operator: None}, extension_id));
    }
    if let Some(operation_id) = opt.operation_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::OPERATION_ID), operator: None}, operation_id));
    }
    if let Some(operation_name) = opt.operation_name.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::OPERATION_NAME), operator: None}, operation_name));
    }
    if let Some(operation_parameter) = opt.operation_parameter.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::OPERATION_PARAMETER), operator: None}, operation_parameter));
    }
    if let Some(attachments) = opt.attachments.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ATTACHMENTS), operator: None}, attachments));
    }
    if let Some(job_step_seq) = opt.job_step_seq.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::JOB_STEP_SEQ), operator: None}, job_step_seq));
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

pub struct JobStepRecordBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> JobStepRecordBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> JobStepRecordBaseService<'a> {
        return JobStepRecordBaseService {
            transaction: transaction
        };
    }

    pub async fn read_job_step_record(&self, id: Id) -> Result<Option<JobStepRecord>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_job_step_record).transpose()?);
    }

    pub async fn read_job_step_record_batch(&self, ids: &[Id]) -> Result<Vec<JobStepRecord>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的任务步骤记录id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobStepRecord> = rows.iter().map(extract_job_step_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_job_step_record(&self, job_step_record: &JobStepRecord) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &job_step_record.id),
                    add_val(&mut vals, &job_step_record.org_id),
                    add_val(&mut vals, &job_step_record.job_id),
                    add_val(&mut vals, &job_step_record.environment_id),
                    add_val(&mut vals, &job_step_record.record_id),
                    add_val(&mut vals, &job_step_record.job_step_id),
                    add_val(&mut vals, &job_step_record.step_name),
                    add_val(&mut vals, &job_step_record.step_type),
                    add_val(&mut vals, &job_step_record.step_remark),
                    add_val(&mut vals, &job_step_record.extension_id),
                    add_val(&mut vals, &job_step_record.operation_id),
                    add_val(&mut vals, &job_step_record.operation_name),
                    add_val(&mut vals, &job_step_record.operation_parameter),
                    add_val(&mut vals, &job_step_record.attachments),
                    add_val(&mut vals, &job_step_record.job_step_seq),
                    add_val(&mut vals, &job_step_record.status),
                    add_val(&mut vals, &job_step_record.created_time),
                    add_val(&mut vals, &job_step_record.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_job_step_record_batch(&self, job_step_record_list: &[JobStepRecord]) -> Result<(), ErrNo> {
        if job_step_record_list.is_empty() {
            log::warn!("待批量新增的任务步骤记录集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * job_step_record_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                job_step_record_list.iter().map(|job_step_record|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &job_step_record.id),
                        add_val(&mut vals, &job_step_record.org_id),
                        add_val(&mut vals, &job_step_record.job_id),
                        add_val(&mut vals, &job_step_record.environment_id),
                        add_val(&mut vals, &job_step_record.record_id),
                        add_val(&mut vals, &job_step_record.job_step_id),
                        add_val(&mut vals, &job_step_record.step_name),
                        add_val(&mut vals, &job_step_record.step_type),
                        add_val(&mut vals, &job_step_record.step_remark),
                        add_val(&mut vals, &job_step_record.extension_id),
                        add_val(&mut vals, &job_step_record.operation_id),
                        add_val(&mut vals, &job_step_record.operation_name),
                        add_val(&mut vals, &job_step_record.operation_parameter),
                        add_val(&mut vals, &job_step_record.attachments),
                        add_val(&mut vals, &job_step_record.job_step_seq),
                        add_val(&mut vals, &job_step_record.status),
                        add_val(&mut vals, &job_step_record.created_time),
                        add_val(&mut vals, &job_step_record.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_step_record_full(&self, job_step_record: &JobStepRecord) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::ORG_ID, "=", &add_val(&mut vals, &job_step_record.org_id),
                    ",", properties::JOB_ID, "=", &add_val(&mut vals, &job_step_record.job_id),
                    ",", properties::ENVIRONMENT_ID, "=", &add_val(&mut vals, &job_step_record.environment_id),
                    ",", properties::RECORD_ID, "=", &add_val(&mut vals, &job_step_record.record_id),
                    ",", properties::JOB_STEP_ID, "=", &add_val(&mut vals, &job_step_record.job_step_id),
                    ",", properties::STEP_NAME, "=", &add_val(&mut vals, &job_step_record.step_name),
                    ",", properties::STEP_TYPE, "=", &add_val(&mut vals, &job_step_record.step_type),
                    ",", properties::STEP_REMARK, "=", &add_val(&mut vals, &job_step_record.step_remark),
                    ",", properties::EXTENSION_ID, "=", &add_val(&mut vals, &job_step_record.extension_id),
                    ",", properties::OPERATION_ID, "=", &add_val(&mut vals, &job_step_record.operation_id),
                    ",", properties::OPERATION_NAME, "=", &add_val(&mut vals, &job_step_record.operation_name),
                    ",", properties::OPERATION_PARAMETER, "=", &add_val(&mut vals, &job_step_record.operation_parameter),
                    ",", properties::ATTACHMENTS, "=", &add_val(&mut vals, &job_step_record.attachments),
                    ",", properties::JOB_STEP_SEQ, "=", &add_val(&mut vals, &job_step_record.job_step_seq),
                    ",", properties::STATUS, "=", &add_val(&mut vals, &job_step_record.status),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &job_step_record.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &job_step_record.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &job_step_record.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_step_record(&self, id: Id, changes: &[JobStepRecordProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&JobStepRecordProperty> = changes
            .iter()
            .filter(|change| match change {
                JobStepRecordProperty::Id(_) => false,
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
                JobStepRecordProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                JobStepRecordProperty::OrgId(org_id) => {
                    [properties::ORG_ID.into(), "=".into(), add_val(&mut vals, org_id).into()]
                },
                JobStepRecordProperty::JobId(job_id) => {
                    [properties::JOB_ID.into(), "=".into(), add_val(&mut vals, job_id).into()]
                },
                JobStepRecordProperty::EnvironmentId(environment_id) => {
                    [properties::ENVIRONMENT_ID.into(), "=".into(), add_val(&mut vals, environment_id).into()]
                },
                JobStepRecordProperty::RecordId(record_id) => {
                    [properties::RECORD_ID.into(), "=".into(), add_val(&mut vals, record_id).into()]
                },
                JobStepRecordProperty::JobStepId(job_step_id) => {
                    [properties::JOB_STEP_ID.into(), "=".into(), add_val(&mut vals, job_step_id).into()]
                },
                JobStepRecordProperty::StepName(step_name) => {
                    [properties::STEP_NAME.into(), "=".into(), add_val(&mut vals, step_name).into()]
                },
                JobStepRecordProperty::StepType(step_type) => {
                    [properties::STEP_TYPE.into(), "=".into(), add_val(&mut vals, step_type).into()]
                },
                JobStepRecordProperty::StepRemark(step_remark) => {
                    [properties::STEP_REMARK.into(), "=".into(), add_val(&mut vals, step_remark).into()]
                },
                JobStepRecordProperty::ExtensionId(extension_id) => {
                    [properties::EXTENSION_ID.into(), "=".into(), add_val(&mut vals, extension_id).into()]
                },
                JobStepRecordProperty::OperationId(operation_id) => {
                    [properties::OPERATION_ID.into(), "=".into(), add_val(&mut vals, operation_id).into()]
                },
                JobStepRecordProperty::OperationName(operation_name) => {
                    [properties::OPERATION_NAME.into(), "=".into(), add_val(&mut vals, operation_name).into()]
                },
                JobStepRecordProperty::OperationParameter(operation_parameter) => {
                    [properties::OPERATION_PARAMETER.into(), "=".into(), add_val(&mut vals, operation_parameter).into()]
                },
                JobStepRecordProperty::Attachments(attachments) => {
                    [properties::ATTACHMENTS.into(), "=".into(), add_val(&mut vals, attachments).into()]
                },
                JobStepRecordProperty::JobStepSeq(job_step_seq) => {
                    [properties::JOB_STEP_SEQ.into(), "=".into(), add_val(&mut vals, job_step_seq).into()]
                },
                JobStepRecordProperty::Status(status) => {
                    [properties::STATUS.into(), "=".into(), add_val(&mut vals, status).into()]
                },
                JobStepRecordProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                JobStepRecordProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_job_step_record(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_job_step_record_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的任务步骤记录id集合为空");
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

    pub async fn query_job_step_record_count(&self, opt: &JobStepRecordOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_job_step_record(&self, page_no: u64, page_size: u64, opt: &JobStepRecordOpt) -> Result<Vec<JobStepRecord>, ErrNo> {
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
        let list: Vec<JobStepRecord> = rows.iter().map(extract_job_step_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_job_step_record_one(&self, opt: &JobStepRecordOpt) -> Result<Option<JobStepRecord>, ErrNo> {
        let list = self.query_job_step_record(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_job_step_record_batch(&self, opt: &JobStepRecordOpt) -> Result<Vec<JobStepRecord>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobStepRecord> = rows.iter().map(extract_job_step_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}