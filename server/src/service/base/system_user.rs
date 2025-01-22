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
use crate::model::system_user::properties;
use crate::model::system_user::SystemUser;
use crate::model::system_user::SystemUserProperty;
use crate::model::system_user::SystemUserOpt;
use crate::native_common;

const ENTITY: &str = "\"system_user\"";
const EXTRA_PROPERTIES: [&str; 5] = [properties::EMAIL,properties::USER_RANDOM_VALUE,properties::HASHED_AUTH_KEY,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_system_user(row: &Row) -> Result<SystemUser, ErrNo> {
    return Ok(SystemUser {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        email: row.try_get(properties::EMAIL).map_err(extract_data_error)?,
        user_random_value: row.try_get(properties::USER_RANDOM_VALUE).map_err(extract_data_error)?,
        hashed_auth_key: row.try_get(properties::HASHED_AUTH_KEY).map_err(extract_data_error)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a SystemUserOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
    let mut pairs = Vec::<(Condition,&(dyn ToSql + std::marker::Sync))>::new();
    if let Some(id) = opt.id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ID), operator: None}, id));
    }
    if let Some(email) = opt.email.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::EMAIL), operator: None}, email));
    }
    if let Some(user_random_value) = opt.user_random_value.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::USER_RANDOM_VALUE), operator: None}, user_random_value));
    }
    if let Some(hashed_auth_key) = opt.hashed_auth_key.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::HASHED_AUTH_KEY), operator: None}, hashed_auth_key));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::CREATED_TIME), operator: None}, created_time));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::LAST_MODIFIED_TIME), operator: None}, last_modified_time));
    }
    return pairs;
}

pub struct SystemUserBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> SystemUserBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> SystemUserBaseService<'a> {
        return SystemUserBaseService {
            transaction: transaction
        };
    }

    pub async fn read_system_user(&self, id: Id) -> Result<Option<SystemUser>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_system_user).transpose()?);
    }

    pub async fn read_system_user_batch(&self, ids: &[Id]) -> Result<Vec<SystemUser>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的系统用户id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<SystemUser> = rows.iter().map(extract_system_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_system_user(&self, system_user: &SystemUser) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &system_user.id),
                    add_val(&mut vals, &system_user.email),
                    add_val(&mut vals, &system_user.user_random_value),
                    add_val(&mut vals, &system_user.hashed_auth_key),
                    add_val(&mut vals, &system_user.created_time),
                    add_val(&mut vals, &system_user.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_system_user_batch(&self, system_user_list: &[SystemUser]) -> Result<(), ErrNo> {
        if system_user_list.is_empty() {
            log::warn!("待批量新增的系统用户集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * system_user_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                system_user_list.iter().map(|system_user|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &system_user.id),
                        add_val(&mut vals, &system_user.email),
                        add_val(&mut vals, &system_user.user_random_value),
                        add_val(&mut vals, &system_user.hashed_auth_key),
                        add_val(&mut vals, &system_user.created_time),
                        add_val(&mut vals, &system_user.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_system_user_full(&self, system_user: &SystemUser) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::EMAIL, "=", &add_val(&mut vals, &system_user.email),
                    ",", properties::USER_RANDOM_VALUE, "=", &add_val(&mut vals, &system_user.user_random_value),
                    ",", properties::HASHED_AUTH_KEY, "=", &add_val(&mut vals, &system_user.hashed_auth_key),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &system_user.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &system_user.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &system_user.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_system_user(&self, id: Id, changes: &[SystemUserProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&SystemUserProperty> = changes
            .iter()
            .filter(|change| match change {
                SystemUserProperty::Id(_) => false,
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
                SystemUserProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                SystemUserProperty::Email(email) => {
                    [properties::EMAIL.into(), "=".into(), add_val(&mut vals, email).into()]
                },
                SystemUserProperty::UserRandomValue(user_random_value) => {
                    [properties::USER_RANDOM_VALUE.into(), "=".into(), add_val(&mut vals, user_random_value).into()]
                },
                SystemUserProperty::HashedAuthKey(hashed_auth_key) => {
                    [properties::HASHED_AUTH_KEY.into(), "=".into(), add_val(&mut vals, hashed_auth_key).into()]
                },
                SystemUserProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                SystemUserProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_system_user(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_system_user_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的系统用户id集合为空");
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

    pub async fn query_system_user_count(&self, opt: &SystemUserOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_system_user(&self, page_no: u64, page_size: u64, opt: &SystemUserOpt) -> Result<Vec<SystemUser>, ErrNo> {
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
        let list: Vec<SystemUser> = rows.iter().map(extract_system_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_system_user_one(&self, opt: &SystemUserOpt) -> Result<Option<SystemUser>, ErrNo> {
        let list = self.query_system_user(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_system_user_batch(&self, opt: &SystemUserOpt) -> Result<Vec<SystemUser>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<SystemUser> = rows.iter().map(extract_system_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}