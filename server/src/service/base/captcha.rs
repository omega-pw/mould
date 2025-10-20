use tihu_native::errno::execute_error;
use tihu_native::errno::extract_data_error;
use tihu_native::errno::prepare_statement_error;
use tihu_native::errno::query_error;
use tihu_native::errno::undefined_enum_value;
use tihu_native::ErrNo;
use tihu::Id;
use tihu::SharedString;
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
use crate::model::captcha::properties;
use crate::model::captcha::Captcha;
use crate::model::captcha::CaptchaProperty;
use crate::model::captcha::CaptchaOpt;
use crate::model::captcha::enums::try_i16_to_receiver_type;
use crate::model::captcha::enums::try_i16_to_scene;
use crate::native_common;

const ENTITY: &str = "captcha";
const EXTRA_PROPERTIES: [&str; 6] = [properties::RECEIVER_TYPE,properties::RECEIVER,properties::SCENE,properties::CAPTCHA,properties::CREATED_TIME,properties::LAST_MODIFIED_TIME,];
const PROPERTY_COUNT: usize = EXTRA_PROPERTIES.len()+1;

fn gen_properties() -> String {
    let properties:Vec<&str> = [properties::ID].iter().chain(EXTRA_PROPERTIES.iter()).map(|item|*item).collect();
    return properties.join(",");
}

lazy_static::lazy_static! {
    static ref PROPERTIES: String = gen_properties();
}

fn extract_captcha(row: &Row) -> Result<Captcha, ErrNo> {
    return Ok(Captcha {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        receiver_type: try_i16_to_receiver_type(row.try_get(properties::RECEIVER_TYPE).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        receiver: row.try_get(properties::RECEIVER).map_err(extract_data_error)?,
        scene: try_i16_to_scene(row.try_get(properties::SCENE).map_err(extract_data_error)?).map_err(undefined_enum_value)?,
        captcha: row.try_get(properties::CAPTCHA).map_err(extract_data_error)?,
        created_time: row.try_get(properties::CREATED_TIME).map_err(extract_data_error)?,
        last_modified_time: row.try_get(properties::LAST_MODIFIED_TIME).map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(opt: &'a CaptchaOpt) -> Vec::<(Condition, &'a (dyn ToSql + std::marker::Sync))> {
    let mut pairs = Vec::<(Condition,&(dyn ToSql + std::marker::Sync))>::new();
    if let Some(id) = opt.id.as_ref() {
        pairs.push((Condition {field: SharedString::from_static(properties::ID), operator: None}, id));
    }
    if let Some(receiver_type) = opt.receiver_type.as_ref() {
        pairs.push((Condition {field: SharedString::from_static(properties::RECEIVER_TYPE), operator: None}, receiver_type));
    }
    if let Some(receiver) = opt.receiver.as_ref() {
        pairs.push((Condition {field: SharedString::from_static(properties::RECEIVER), operator: None}, receiver));
    }
    if let Some(scene) = opt.scene.as_ref() {
        pairs.push((Condition {field: SharedString::from_static(properties::SCENE), operator: None}, scene));
    }
    if let Some(captcha) = opt.captcha.as_ref() {
        pairs.push((Condition {field: SharedString::from_static(properties::CAPTCHA), operator: None}, captcha));
    }
    if let Some(created_time) = opt.created_time.as_ref() {
        pairs.push((Condition {field: SharedString::from_static(properties::CREATED_TIME), operator: None}, created_time));
    }
    if let Some(last_modified_time) = opt.last_modified_time.as_ref() {
        pairs.push((Condition {field: SharedString::from_static(properties::LAST_MODIFIED_TIME), operator: None}, last_modified_time));
    }
    return pairs;
}

pub struct CaptchaBaseService<'a> {
    transaction: &'a Transaction<'a>
}

impl<'a> CaptchaBaseService<'a> {

    pub fn new(transaction: &'a Transaction) -> CaptchaBaseService<'a> {
        return CaptchaBaseService {
            transaction: transaction
        };
    }

    pub async fn read_captcha(&self, id: Id) -> Result<Option<Captcha>, ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let row_opt: Option<Row> = self.transaction.query_opt(&statement, &vals).await.map_err(query_error)?;
        return Ok(row_opt.as_ref().map(extract_captcha).transpose()?);
    }

    pub async fn read_captcha_batch(&self, ids: &[Id]) -> Result<Vec<Captcha>, ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量读取的验证码id集合为空");
            return Ok(Vec::new());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(ids.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}" where "{properties::ID}" in (" {add_vals(&mut vals, &ids)} ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<Captcha> = rows.iter().map(extract_captcha).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn insert_captcha(&self, captcha: &Captcha) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values(" {
                vec![
                    add_val(&mut vals, &captcha.id),
                    add_val(&mut vals, &captcha.receiver_type),
                    add_val(&mut vals, &captcha.receiver),
                    add_val(&mut vals, &captcha.scene),
                    add_val(&mut vals, &captcha.captcha),
                    add_val(&mut vals, &captcha.created_time),
                    add_val(&mut vals, &captcha.last_modified_time),
                ].join(",")
            } ")"
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn insert_captcha_batch(&self, captcha_list: &[Captcha]) -> Result<(), ErrNo> {
        if captcha_list.is_empty() {
            log::warn!("待批量新增的验证码集合为空");
            return Ok(());
        }
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT * captcha_list.len());
        let sql = format_xml::template! {
            "insert into "{ENTITY}"(" {PROPERTIES.as_str()} ") values" {
                captcha_list.iter().map(|captcha|{
                    let trunks:Vec<String> = vec![
                        add_val(&mut vals, &captcha.id),
                        add_val(&mut vals, &captcha.receiver_type),
                        add_val(&mut vals, &captcha.receiver),
                        add_val(&mut vals, &captcha.scene),
                        add_val(&mut vals, &captcha.captcha),
                        add_val(&mut vals, &captcha.created_time),
                        add_val(&mut vals, &captcha.last_modified_time),
                    ];
                    ["(", &trunks.join(","), ")"].concat()
                }).collect::<Vec<String>>().join(",")
            }
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_captcha_full(&self, captcha: &Captcha) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(PROPERTY_COUNT);
        let sql = format_xml::template! {
            "update "{ENTITY}" set " {
                [
                    properties::RECEIVER_TYPE, "=", &add_val(&mut vals, &captcha.receiver_type),
                    ",", properties::RECEIVER, "=", &add_val(&mut vals, &captcha.receiver),
                    ",", properties::SCENE, "=", &add_val(&mut vals, &captcha.scene),
                    ",", properties::CAPTCHA, "=", &add_val(&mut vals, &captcha.captcha),
                    ",", properties::CREATED_TIME, "=", &add_val(&mut vals, &captcha.created_time),
                    ",", properties::LAST_MODIFIED_TIME, "=", &add_val(&mut vals, &captcha.last_modified_time),
                ].concat()
            } " where "{properties::ID}"=" {add_val(&mut vals, &captcha.id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn update_captcha(&self, id: Id, changes: &[CaptchaProperty]) -> Result<(), ErrNo> {
        let changes: Vec<&CaptchaProperty> = changes
            .iter()
            .filter(|change| match change {
                CaptchaProperty::Id(_) => false,
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
                CaptchaProperty::Id(id) => {
                    [properties::ID.into(), "=".into(), add_val(&mut vals, id).into()]
                },
                CaptchaProperty::ReceiverType(receiver_type) => {
                    [properties::RECEIVER_TYPE.into(), "=".into(), add_val(&mut vals, receiver_type).into()]
                },
                CaptchaProperty::Receiver(receiver) => {
                    [properties::RECEIVER.into(), "=".into(), add_val(&mut vals, receiver).into()]
                },
                CaptchaProperty::Scene(scene) => {
                    [properties::SCENE.into(), "=".into(), add_val(&mut vals, scene).into()]
                },
                CaptchaProperty::Captcha(captcha) => {
                    [properties::CAPTCHA.into(), "=".into(), add_val(&mut vals, captcha).into()]
                },
                CaptchaProperty::CreatedTime(created_time) => {
                    [properties::CREATED_TIME.into(), "=".into(), add_val(&mut vals, created_time).into()]
                },
                CaptchaProperty::LastModifiedTime(last_modified_time) => {
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

    pub async fn delete_captcha(&self, id: Id) -> Result<(), ErrNo> {
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(1);
        let sql = format_xml::template! {
            "delete from "{ENTITY}" where "{properties::ID}"="{add_val(&mut vals, &id)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        self.transaction.execute(&statement, &vals).await.map_err(execute_error)?;
        return Ok(());
    }

    pub async fn delete_captcha_batch(&self, ids: &[Id]) -> Result<(), ErrNo>
    {
        if ids.is_empty() {
            log::warn!("待批量删除的验证码id集合为空");
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

    pub async fn query_captcha_count(&self, opt: &CaptchaOpt) -> Result<u64, ErrNo> {
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

    pub async fn query_captcha(&self, page_no: u64, page_size: u64, opt: &CaptchaOpt) -> Result<Vec<Captcha>, ErrNo> {
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
        let list: Vec<Captcha> = rows.iter().map(extract_captcha).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_captcha_one(&self, opt: &CaptchaOpt) -> Result<Option<Captcha>, ErrNo> {
        let list = self.query_captcha(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_captcha_batch(&self, opt: &CaptchaOpt) -> Result<Vec<Captcha>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{PROPERTIES.as_str()}" from "{ENTITY}
            {if pairs.is_empty() {""} else {" where "}}
            {add_conditions(&mut vals, &pairs)}
        }.to_string();
        let statement = self.transaction.prepare(&sql).await.map_err(prepare_statement_error)?;
        let rows: Vec<Row> = self.transaction.query(&statement, &vals).await.map_err(query_error)?;
        let list: Vec<Captcha> = rows.iter().map(extract_captcha).collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

}