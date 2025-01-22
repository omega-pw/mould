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
use crate::model::external_user::properties;
use crate::model::external_user::ExternalUser;
use crate::model::external_user::ExternalUserProperty;
use crate::model::external_user::ExternalUserOpt;
use crate::model::external_user::enums::try_i16_to_provider_type;
use crate::native_common;

const ENTITY: &str = "external_user";
const EXTRA_PROPERTIES: [&str; 6] = [properties::PROVIDER_TYPE,properties::PROVIDER,properties::OPENID,properties::DETAIL,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_external_user(row: &Row) -> Result<ExternalUser, ErrNo> {
    return Ok(ExternalUser {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        provider_type: try_i16_to_provider_type(row.try_get(properties::PROVIDER_TYPE).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        provider: row.try_get(properties::PROVIDER).map_err(extract_data_error)?,
        openid: row.try_get(properties::OPENID).map_err(extract_data_error)?,
        detail: row.try_get(properties::DETAIL).map_err(extract_data_error)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a ExternalUserOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
    let mut pairs = Vec::<(Condition,&(dyn ToSql + std::marker::Sync))>::new();
    if let Some(id) = opt.id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ID), operator: None}, id));
    }
    if let Some(provider_type) = opt.provider_type.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::PROVIDER_TYPE), operator: None}, provider_type));
    }
    if let Some(provider) = opt.provider.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::PROVIDER), operator: None}, provider));
    }
    if let Some(openid) = opt.openid.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::OPENID), operator: None}, openid));
    }
    if let Some(detail) = opt.detail.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::DETAIL), operator: None}, detail));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::CREATED_TIME), operator: None}, created_time));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::LAST_MODIFIED_TIME), operator: None}, last_modified_time));
    }
    return pairs;
}

pub struct ExternalUserBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> ExternalUserBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> ExternalUserBaseService<'a> {
        return ExternalUserBaseService {
            transaction: transaction
        };
    }

    pub async fn read_external_user(&self, id: Id) -> Result<Option<ExternalUser>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_external_user).transpose()?);
    }

    pub async fn read_external_user_batch(&self, ids: &[Id]) -> Result<Vec<ExternalUser>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的外部用户id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<ExternalUser> = rows.iter().map(extract_external_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_external_user(&self, external_user: &ExternalUser) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &external_user.id),
                    add_val(&mut vals, &external_user.provider_type),
                    add_val(&mut vals, &external_user.provider),
                    add_val(&mut vals, &external_user.openid),
                    add_val(&mut vals, &external_user.detail),
                    add_val(&mut vals, &external_user.created_time),
                    add_val(&mut vals, &external_user.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_external_user_batch(&self, external_user_list: &[ExternalUser]) -> Result<(), ErrNo> {
        if external_user_list.is_empty() {
            log::warn!("待批量新增的外部用户集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * external_user_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                external_user_list.iter().map(|external_user|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &external_user.id),
                        add_val(&mut vals, &external_user.provider_type),
                        add_val(&mut vals, &external_user.provider),
                        add_val(&mut vals, &external_user.openid),
                        add_val(&mut vals, &external_user.detail),
                        add_val(&mut vals, &external_user.created_time),
                        add_val(&mut vals, &external_user.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_external_user_full(&self, external_user: &ExternalUser) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::PROVIDER_TYPE, "=", &add_val(&mut vals, &external_user.provider_type),
                    ",", properties::PROVIDER, "=", &add_val(&mut vals, &external_user.provider),
                    ",", properties::OPENID, "=", &add_val(&mut vals, &external_user.openid),
                    ",", properties::DETAIL, "=", &add_val(&mut vals, &external_user.detail),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &external_user.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &external_user.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &external_user.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_external_user(&self, id: Id, changes: &[ExternalUserProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&ExternalUserProperty> = changes
            .iter()
            .filter(|change| match change {
                ExternalUserProperty::Id(_) => false,
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
                ExternalUserProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                ExternalUserProperty::ProviderType(provider_type) => {
                    [properties::PROVIDER_TYPE.into(), "=".into(), add_val(&mut vals, provider_type).into()]
                },
                ExternalUserProperty::Provider(provider) => {
                    [properties::PROVIDER.into(), "=".into(), add_val(&mut vals, provider).into()]
                },
                ExternalUserProperty::Openid(openid) => {
                    [properties::OPENID.into(), "=".into(), add_val(&mut vals, openid).into()]
                },
                ExternalUserProperty::Detail(detail) => {
                    [properties::DETAIL.into(), "=".into(), add_val(&mut vals, detail).into()]
                },
                ExternalUserProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                ExternalUserProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_external_user(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_external_user_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的外部用户id集合为空");
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

    pub async fn query_external_user_count(&self, opt: &ExternalUserOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_external_user(&self, page_no: u64, page_size: u64, opt: &ExternalUserOpt) -> Result<Vec<ExternalUser>, ErrNo> {
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
        let list: Vec<ExternalUser> = rows.iter().map(extract_external_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_external_user_one(&self, opt: &ExternalUserOpt) -> Result<Option<ExternalUser>, ErrNo> {
        let list = self.query_external_user(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_external_user_batch(&self, opt: &ExternalUserOpt) -> Result<Vec<ExternalUser>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<ExternalUser> = rows.iter().map(extract_external_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}