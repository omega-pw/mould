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
use crate::model::user::properties;
use crate::model::user::User;
use crate::model::user::UserProperty;
use crate::model::user::UserOpt;
use crate::model::user::enums::try_i16_to_user_source;
use crate::native_common;

const ENTITY: &str = "\"user\"";
const EXTRA_PROPERTIES: [&str; 6] = [properties::ORG_ID,properties::USER_SOURCE,properties::NAME,properties::AVATAR_URL,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_user(row: &Row) -> Result<User, ErrNo> {
    return Ok(User {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        org_id: row.try_get(properties::ORG_ID).map_err(extract_data_error)?,
        user_source: try_i16_to_user_source(row.try_get(properties::USER_SOURCE).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        name: row.try_get(properties::NAME).map_err(extract_data_error)?,
        avatar_url: row.try_get(properties::AVATAR_URL).map_err(extract_data_error)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a UserOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
    let mut pairs = Vec::<(Condition,&(dyn ToSql + std::marker::Sync))>::new();
    if let Some(id) = opt.id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ID), operator: None}, id));
    }
    if let Some(org_id) = opt.org_id.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::ORG_ID), operator: None}, org_id));
    }
    if let Some(user_source) = opt.user_source.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::USER_SOURCE), operator: None}, user_source));
    }
    if let Some(name) = opt.name.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::NAME), operator: None}, name));
    }
    if let Some(avatar_url) = opt.avatar_url.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::AVATAR_URL), operator: None}, avatar_url));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::CREATED_TIME), operator: None}, created_time));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((Condition {field: LightString::from_static(properties::LAST_MODIFIED_TIME), operator: None}, last_modified_time));
    }
    return pairs;
}

pub struct UserBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> UserBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> UserBaseService<'a> {
        return UserBaseService {
            transaction: transaction
        };
    }

    pub async fn read_user(&self, id: Id) -> Result<Option<User>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_user).transpose()?);
    }

    pub async fn read_user_batch(&self, ids: &[Id]) -> Result<Vec<User>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的用户id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<User> = rows.iter().map(extract_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_user(&self, user: &User) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &user.id),
                    add_val(&mut vals, &user.org_id),
                    add_val(&mut vals, &user.user_source),
                    add_val(&mut vals, &user.name),
                    add_val(&mut vals, &user.avatar_url),
                    add_val(&mut vals, &user.created_time),
                    add_val(&mut vals, &user.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_user_batch(&self, user_list: &[User]) -> Result<(), ErrNo> {
        if user_list.is_empty() {
            log::warn!("待批量新增的用户集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * user_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                user_list.iter().map(|user|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &user.id),
                        add_val(&mut vals, &user.org_id),
                        add_val(&mut vals, &user.user_source),
                        add_val(&mut vals, &user.name),
                        add_val(&mut vals, &user.avatar_url),
                        add_val(&mut vals, &user.created_time),
                        add_val(&mut vals, &user.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_user_full(&self, user: &User) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::ORG_ID, "=", &add_val(&mut vals, &user.org_id),
                    ",", properties::USER_SOURCE, "=", &add_val(&mut vals, &user.user_source),
                    ",", properties::NAME, "=", &add_val(&mut vals, &user.name),
                    ",", properties::AVATAR_URL, "=", &add_val(&mut vals, &user.avatar_url),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &user.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &user.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &user.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_user(&self, id: Id, changes: &[UserProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&UserProperty> = changes
            .iter()
            .filter(|change| match change {
                UserProperty::Id(_) => false,
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
                UserProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                UserProperty::OrgId(org_id) => {
                    [properties::ORG_ID.into(), "=".into(), add_val(&mut vals, org_id).into()]
                },
                UserProperty::UserSource(user_source) => {
                    [properties::USER_SOURCE.into(), "=".into(), add_val(&mut vals, user_source).into()]
                },
                UserProperty::Name(name) => {
                    [properties::NAME.into(), "=".into(), add_val(&mut vals, name).into()]
                },
                UserProperty::AvatarUrl(avatar_url) => {
                    [properties::AVATAR_URL.into(), "=".into(), add_val(&mut vals, avatar_url).into()]
                },
                UserProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                UserProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_user(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_user_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的用户id集合为空");
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

    pub async fn query_user_count(&self, opt: &UserOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_user(&self, page_no: u64, page_size: u64, opt: &UserOpt) -> Result<Vec<User>, ErrNo> {
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
        let list: Vec<User> = rows.iter().map(extract_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_user_one(&self, opt: &UserOpt) -> Result<Option<User>, ErrNo> {
        let list = self.query_user(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_user_batch(&self, opt: &UserOpt) -> Result<Vec<User>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<User> = rows.iter().map(extract_user).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}