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
use crate::model::job_step_resource_record::properties;
use crate::model::job_step_resource_record::JobStepResourceRecord;
use crate::model::job_step_resource_record::JobStepResourceRecordProperty;
use crate::model::job_step_resource_record::JobStepResourceRecordOpt;
use crate::model::job_step_resource_record::enums::try_i16_to_status;
use crate::native_common;

const ENTITY: &str = "job_step_resource_record";
const EXTRA_PROPERTIES: [&str; 13] = [properties::ORG_ID,properties::JOB_ID,properties::ENVIRONMENT_ID,properties::RECORD_ID,properties::JOB_STEP_RECORD_ID,properties::ENVIRONMENT_RESOURCE_ID,properties::RESOURCE_NAME,properties::EXTENSION_CONFIGURATION,properties::OUTPUT_FILE,properties::OUTPUT_CONTENT,properties::STATUS,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_job_step_resource_record(row: &Row) -> Result<JobStepResourceRecord, ErrNo> {
    return Ok(JobStepResourceRecord {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row.try_get(properties::ORG_ID).map_err(extract_data_error)?,
        job_id: row.try_get(properties::JOB_ID).map_err(extract_data_error)?,
        environment_id: row.try_get(properties::ENVIRONMENT_ID).map_err(extract_data_error)?,
        record_id: row.try_get(properties::RECORD_ID).map_err(extract_data_error)?,
        job_step_record_id: row.try_get(properties::JOB_STEP_RECORD_ID).map_err(extract_data_error)?,
        environment_resource_id: row.try_get(properties::ENVIRONMENT_RESOURCE_ID).map_err(extract_data_error)?,
        resource_name: row.try_get(properties::RESOURCE_NAME).map_err(extract_data_error)?,
        extension_configuration: row.try_get(properties::EXTENSION_CONFIGURATION).map_err(extract_data_error)?,
        output_file: row.try_get(properties::OUTPUT_FILE).map_err(extract_data_error)?,
        output_content: row.try_get(properties::OUTPUT_CONTENT).map_err(extract_data_error)?,
        status: try_i16_to_status(row.try_get(properties::STATUS).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a JobStepResourceRecordOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
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
    if let Some(job_step_record_id) = opt.job_step_record_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::JOB_STEP_RECORD_ID), operator: None}, job_step_record_id));
    }
    if let Some(environment_resource_id) = opt.environment_resource_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ENVIRONMENT_RESOURCE_ID), operator: None}, environment_resource_id));
    }
    if let Some(resource_name) = opt.resource_name.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::RESOURCE_NAME), operator: None}, resource_name));
    }
    if let Some(extension_configuration) = opt.extension_configuration.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::EXTENSION_CONFIGURATION), operator: None}, extension_configuration));
    }
    if let Some(output_file) = opt.output_file.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::OUTPUT_FILE), operator: None}, output_file));
    }
    if let Some(output_content) = opt.output_content.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::OUTPUT_CONTENT), operator: None}, output_content));
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

pub struct JobStepResourceRecordBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> JobStepResourceRecordBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> JobStepResourceRecordBaseService<'a> {
        return JobStepResourceRecordBaseService {
            transaction: transaction
        };
    }

    pub async fn read_job_step_resource_record(&self, id: Id) -> Result<Option<JobStepResourceRecord>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_job_step_resource_record).transpose()?);
    }

    pub async fn read_job_step_resource_record_batch(&self, ids: &[Id]) -> Result<Vec<JobStepResourceRecord>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的任务步骤资源记录id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobStepResourceRecord> = rows.iter().map(extract_job_step_resource_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_job_step_resource_record(&self, job_step_resource_record: &JobStepResourceRecord) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &job_step_resource_record.id),
                    add_val(&mut vals, &job_step_resource_record.org_id),
                    add_val(&mut vals, &job_step_resource_record.job_id),
                    add_val(&mut vals, &job_step_resource_record.environment_id),
                    add_val(&mut vals, &job_step_resource_record.record_id),
                    add_val(&mut vals, &job_step_resource_record.job_step_record_id),
                    add_val(&mut vals, &job_step_resource_record.environment_resource_id),
                    add_val(&mut vals, &job_step_resource_record.resource_name),
                    add_val(&mut vals, &job_step_resource_record.extension_configuration),
                    add_val(&mut vals, &job_step_resource_record.output_file),
                    add_val(&mut vals, &job_step_resource_record.output_content),
                    add_val(&mut vals, &job_step_resource_record.status),
                    add_val(&mut vals, &job_step_resource_record.created_time),
                    add_val(&mut vals, &job_step_resource_record.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_job_step_resource_record_batch(&self, job_step_resource_record_list: &[JobStepResourceRecord]) -> Result<(), ErrNo> {
        if job_step_resource_record_list.is_empty() {
            log::warn!("待批量新增的任务步骤资源记录集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * job_step_resource_record_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                job_step_resource_record_list.iter().map(|job_step_resource_record|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &job_step_resource_record.id),
                        add_val(&mut vals, &job_step_resource_record.org_id),
                        add_val(&mut vals, &job_step_resource_record.job_id),
                        add_val(&mut vals, &job_step_resource_record.environment_id),
                        add_val(&mut vals, &job_step_resource_record.record_id),
                        add_val(&mut vals, &job_step_resource_record.job_step_record_id),
                        add_val(&mut vals, &job_step_resource_record.environment_resource_id),
                        add_val(&mut vals, &job_step_resource_record.resource_name),
                        add_val(&mut vals, &job_step_resource_record.extension_configuration),
                        add_val(&mut vals, &job_step_resource_record.output_file),
                        add_val(&mut vals, &job_step_resource_record.output_content),
                        add_val(&mut vals, &job_step_resource_record.status),
                        add_val(&mut vals, &job_step_resource_record.created_time),
                        add_val(&mut vals, &job_step_resource_record.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_step_resource_record_full(&self, job_step_resource_record: &JobStepResourceRecord) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::ORG_ID, "=", &add_val(&mut vals, &job_step_resource_record.org_id),
                    ",", properties::JOB_ID, "=", &add_val(&mut vals, &job_step_resource_record.job_id),
                    ",", properties::ENVIRONMENT_ID, "=", &add_val(&mut vals, &job_step_resource_record.environment_id),
                    ",", properties::RECORD_ID, "=", &add_val(&mut vals, &job_step_resource_record.record_id),
                    ",", properties::JOB_STEP_RECORD_ID, "=", &add_val(&mut vals, &job_step_resource_record.job_step_record_id),
                    ",", properties::ENVIRONMENT_RESOURCE_ID, "=", &add_val(&mut vals, &job_step_resource_record.environment_resource_id),
                    ",", properties::RESOURCE_NAME, "=", &add_val(&mut vals, &job_step_resource_record.resource_name),
                    ",", properties::EXTENSION_CONFIGURATION, "=", &add_val(&mut vals, &job_step_resource_record.extension_configuration),
                    ",", properties::OUTPUT_FILE, "=", &add_val(&mut vals, &job_step_resource_record.output_file),
                    ",", properties::OUTPUT_CONTENT, "=", &add_val(&mut vals, &job_step_resource_record.output_content),
                    ",", properties::STATUS, "=", &add_val(&mut vals, &job_step_resource_record.status),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &job_step_resource_record.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &job_step_resource_record.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &job_step_resource_record.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_job_step_resource_record(&self, id: Id, changes: &[JobStepResourceRecordProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&JobStepResourceRecordProperty> = changes
            .iter()
            .filter(|change| match change {
                JobStepResourceRecordProperty::Id(_) => false,
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
                JobStepResourceRecordProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                JobStepResourceRecordProperty::OrgId(org_id) => {
                    [properties::ORG_ID.into(), "=".into(), add_val(&mut vals, org_id).into()]
                },
                JobStepResourceRecordProperty::JobId(job_id) => {
                    [properties::JOB_ID.into(), "=".into(), add_val(&mut vals, job_id).into()]
                },
                JobStepResourceRecordProperty::EnvironmentId(environment_id) => {
                    [properties::ENVIRONMENT_ID.into(), "=".into(), add_val(&mut vals, environment_id).into()]
                },
                JobStepResourceRecordProperty::RecordId(record_id) => {
                    [properties::RECORD_ID.into(), "=".into(), add_val(&mut vals, record_id).into()]
                },
                JobStepResourceRecordProperty::JobStepRecordId(job_step_record_id) => {
                    [properties::JOB_STEP_RECORD_ID.into(), "=".into(), add_val(&mut vals, job_step_record_id).into()]
                },
                JobStepResourceRecordProperty::EnvironmentResourceId(environment_resource_id) => {
                    [properties::ENVIRONMENT_RESOURCE_ID.into(), "=".into(), add_val(&mut vals, environment_resource_id).into()]
                },
                JobStepResourceRecordProperty::ResourceName(resource_name) => {
                    [properties::RESOURCE_NAME.into(), "=".into(), add_val(&mut vals, resource_name).into()]
                },
                JobStepResourceRecordProperty::ExtensionConfiguration(extension_configuration) => {
                    [properties::EXTENSION_CONFIGURATION.into(), "=".into(), add_val(&mut vals, extension_configuration).into()]
                },
                JobStepResourceRecordProperty::OutputFile(output_file) => {
                    [properties::OUTPUT_FILE.into(), "=".into(), add_val(&mut vals, output_file).into()]
                },
                JobStepResourceRecordProperty::OutputContent(output_content) => {
                    [properties::OUTPUT_CONTENT.into(), "=".into(), add_val(&mut vals, output_content).into()]
                },
                JobStepResourceRecordProperty::Status(status) => {
                    [properties::STATUS.into(), "=".into(), add_val(&mut vals, status).into()]
                },
                JobStepResourceRecordProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                JobStepResourceRecordProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_job_step_resource_record(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_job_step_resource_record_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的任务步骤资源记录id集合为空");
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

    pub async fn query_job_step_resource_record_count(&self, opt: &JobStepResourceRecordOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_job_step_resource_record(&self, page_no: u64, page_size: u64, opt: &JobStepResourceRecordOpt) -> Result<Vec<JobStepResourceRecord>, ErrNo> {
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
        let list: Vec<JobStepResourceRecord> = rows.iter().map(extract_job_step_resource_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_job_step_resource_record_one(&self, opt: &JobStepResourceRecordOpt) -> Result<Option<JobStepResourceRecord>, ErrNo> {
        let list = self.query_job_step_resource_record(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_job_step_resource_record_batch(&self, opt: &JobStepResourceRecordOpt) -> Result<Vec<JobStepResourceRecord>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<JobStepResourceRecord> = rows.iter().map(extract_job_step_resource_record).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}