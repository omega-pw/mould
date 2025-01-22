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
use crate::model::job_step::properties;
use crate::model::job_step::JobStep;
use crate::model::job_step::JobStepProperty;
use crate::model::job_step::JobStepOpt;
use crate::model::job_step::enums::try_i16_to_step_type;
use crate::native_common;

const ENTITY: &str = "job_step";
const EXTRA_PROPERTIES: [&str; 13] = [properties::ORG_ID,properties::JOB_ID,properties::NAME,properties::STEP_TYPE,properties::SCHEMA_RESOURCE_ID,properties::OPERATION_ID,properties::OPERATION_NAME,properties::OPERATION_PARAMETER,properties::ATTACHMENTS,properties::REMARK,properties::SEQ,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_job_step(row: &Row) -> Result<JobStep, ErrNo> {
    return Ok(JobStep {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row.try_get(properties::ORG_ID).map_err(extract_data_error)?,
        job_id: row.try_get(properties::JOB_ID).map_err(extract_data_error)?,
        name: row.try_get(properties::NAME).map_err(extract_data_error)?,
        step_type: try_i16_to_step_type(row.try_get(properties::STEP_TYPE).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        schema_resource_id: row.try_get(properties::SCHEMA_RESOURCE_ID).map_err(extract_data_error)?,
        operation_id: row.try_get(properties::OPERATION_ID).map_err(extract_data_error)?,
        operation_name: row.try_get(properties::OPERATION_NAME).map_err(extract_data_error)?,
        operation_parameter: row.try_get(properties::OPERATION_PARAMETER).map_err(extract_data_error)?,
        attachments: row.try_get(properties::ATTACHMENTS).map_err(extract_data_error)?,
        remark: row.try_get(properties::REMARK).map_err(extract_data_error)?,
        seq: row.try_get(properties::SEQ).map_err(extract_data_error)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a JobStepOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
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
    if let Some(name) = opt.name.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::NAME), operator: None}, name));
    }
    if let Some(step_type) = opt.step_type.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::STEP_TYPE), operator: None}, step_type));
    }
    if let Some(schema_resource_id) = opt.schema_resource_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::SCHEMA_RESOURCE_ID), operator: None}, schema_resource_id));
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
    if let Some(remark) = opt.remark.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::REMARK), operator: None}, remark));
    }
    if let Some(seq) = opt.seq.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::SEQ), operator: None}, seq));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::CREATED_TIME), operator: None}, created_time));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::LAST_MODIFIED_TIME), operator: None}, last_modified_time));
    }
    return pairs;
}

pub struct JobStepBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> JobStepBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> JobStepBaseService<'a> {
        return JobStepBaseService {
            transaction: transaction
        };
    }

    pub async fn read_job_step(&self, id: Id) -> Result<Option<JobStep>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_job_step).transpose()?);
    }

    pub async fn read_job_step_batch(&self, ids: &[Id]) -> Result<Vec<JobStep>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的任务步骤id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobStep> = rows.iter().map(extract_job_step).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_job_step(&self, job_step: &JobStep) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &job_step.id),
                    add_val(&mut vals, &job_step.org_id),
                    add_val(&mut vals, &job_step.job_id),
                    add_val(&mut vals, &job_step.name),
                    add_val(&mut vals, &job_step.step_type),
                    add_val(&mut vals, &job_step.schema_resource_id),
                    add_val(&mut vals, &job_step.operation_id),
                    add_val(&mut vals, &job_step.operation_name),
                    add_val(&mut vals, &job_step.operation_parameter),
                    add_val(&mut vals, &job_step.attachments),
                    add_val(&mut vals, &job_step.remark),
                    add_val(&mut vals, &job_step.seq),
                    add_val(&mut vals, &job_step.created_time),
                    add_val(&mut vals, &job_step.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_job_step_batch(&self, job_step_list: &[JobStep]) -> Result<(), ErrNo> {
        if job_step_list.is_empty() {
            log::warn!("待批量新增的任务步骤集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * job_step_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                job_step_list.iter().map(|job_step|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &job_step.id),
                        add_val(&mut vals, &job_step.org_id),
                        add_val(&mut vals, &job_step.job_id),
                        add_val(&mut vals, &job_step.name),
                        add_val(&mut vals, &job_step.step_type),
                        add_val(&mut vals, &job_step.schema_resource_id),
                        add_val(&mut vals, &job_step.operation_id),
                        add_val(&mut vals, &job_step.operation_name),
                        add_val(&mut vals, &job_step.operation_parameter),
                        add_val(&mut vals, &job_step.attachments),
                        add_val(&mut vals, &job_step.remark),
                        add_val(&mut vals, &job_step.seq),
                        add_val(&mut vals, &job_step.created_time),
                        add_val(&mut vals, &job_step.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_step_full(&self, job_step: &JobStep) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::ORG_ID, "=", &add_val(&mut vals, &job_step.org_id),
                    ",", properties::JOB_ID, "=", &add_val(&mut vals, &job_step.job_id),
                    ",", properties::NAME, "=", &add_val(&mut vals, &job_step.name),
                    ",", properties::STEP_TYPE, "=", &add_val(&mut vals, &job_step.step_type),
                    ",", properties::SCHEMA_RESOURCE_ID, "=", &add_val(&mut vals, &job_step.schema_resource_id),
                    ",", properties::OPERATION_ID, "=", &add_val(&mut vals, &job_step.operation_id),
                    ",", properties::OPERATION_NAME, "=", &add_val(&mut vals, &job_step.operation_name),
                    ",", properties::OPERATION_PARAMETER, "=", &add_val(&mut vals, &job_step.operation_parameter),
                    ",", properties::ATTACHMENTS, "=", &add_val(&mut vals, &job_step.attachments),
                    ",", properties::REMARK, "=", &add_val(&mut vals, &job_step.remark),
                    ",", properties::SEQ, "=", &add_val(&mut vals, &job_step.seq),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &job_step.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &job_step.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &job_step.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_step(&self, id: Id, changes: &[JobStepProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&JobStepProperty> = changes
            .iter()
            .filter(|change| match change {
                JobStepProperty::Id(_) => false,
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
                JobStepProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                JobStepProperty::OrgId(org_id) => {
                    [properties::ORG_ID.into(), "=".into(), add_val(&mut vals, org_id).into()]
                },
                JobStepProperty::JobId(job_id) => {
                    [properties::JOB_ID.into(), "=".into(), add_val(&mut vals, job_id).into()]
                },
                JobStepProperty::Name(name) => {
                    [properties::NAME.into(), "=".into(), add_val(&mut vals, name).into()]
                },
                JobStepProperty::StepType(step_type) => {
                    [properties::STEP_TYPE.into(), "=".into(), add_val(&mut vals, step_type).into()]
                },
                JobStepProperty::SchemaResourceId(schema_resource_id) => {
                    [properties::SCHEMA_RESOURCE_ID.into(), "=".into(), add_val(&mut vals, schema_resource_id).into()]
                },
                JobStepProperty::OperationId(operation_id) => {
                    [properties::OPERATION_ID.into(), "=".into(), add_val(&mut vals, operation_id).into()]
                },
                JobStepProperty::OperationName(operation_name) => {
                    [properties::OPERATION_NAME.into(), "=".into(), add_val(&mut vals, operation_name).into()]
                },
                JobStepProperty::OperationParameter(operation_parameter) => {
                    [properties::OPERATION_PARAMETER.into(), "=".into(), add_val(&mut vals, operation_parameter).into()]
                },
                JobStepProperty::Attachments(attachments) => {
                    [properties::ATTACHMENTS.into(), "=".into(), add_val(&mut vals, attachments).into()]
                },
                JobStepProperty::Remark(remark) => {
                    [properties::REMARK.into(), "=".into(), add_val(&mut vals, remark).into()]
                },
                JobStepProperty::Seq(seq) => {
                    [properties::SEQ.into(), "=".into(), add_val(&mut vals, seq).into()]
                },
                JobStepProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                JobStepProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_job_step(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_job_step_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的任务步骤id集合为空");
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

    pub async fn query_job_step_count(&self, opt: &JobStepOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_job_step(&self, page_no: u64, page_size: u64, opt: &JobStepOpt) -> Result<Vec<JobStep>, ErrNo> {
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
        let list: Vec<JobStep> = rows.iter().map(extract_job_step).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_job_step_one(&self, opt: &JobStepOpt) -> Result<Option<JobStep>, ErrNo> {
        let list = self.query_job_step(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_job_step_batch(&self, opt: &JobStepOpt) -> Result<Vec<JobStep>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobStep> = rows.iter().map(extract_job_step).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}