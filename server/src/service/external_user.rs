use crate::model::external_user::enums::try_i16_to_provider_type;
use crate::model::external_user::properties;
use crate::model::external_user::ExternalUser;
use crate::model::external_user::ExternalUserOpt;
use crate::native_common;
use format_xml;
use lazy_static;
use native_common::utils::add_conditions;
use native_common::utils::calc_sql_pagination;
use native_common::utils::Condition;
use tihu::LightString;
use tihu_native::errno::extract_data_error;
use tihu_native::errno::prepare_statement_error;
use tihu_native::errno::query_error;
use tihu_native::errno::undefined_enum_value;
use tihu_native::ErrNo;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Row, Transaction};

const ENTITY: &str = "external_user";
const EXTRA_PROPERTIES: [&str; 6] = [
    properties::PROVIDER_TYPE,
    properties::PROVIDER,
    properties::OPENID,
    properties::DETAIL,
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

fn extract_external_user(row: &Row) -> Result<ExternalUser, ErrNo> {
    return Ok(ExternalUser {
        id: row.try_get(properties::ID).map_err(extract_data_error)?,
        provider_type: try_i16_to_provider_type(
            row.try_get(properties::PROVIDER_TYPE)
                .map_err(extract_data_error)?,
        )
        .map_err(undefined_enum_value)?,
        provider: row
            .try_get(properties::PROVIDER)
            .map_err(extract_data_error)?,
        openid: row
            .try_get(properties::OPENID)
            .map_err(extract_data_error)?,
        detail: row
            .try_get(properties::DETAIL)
            .map_err(extract_data_error)?,
        created_time: row
            .try_get(properties::CREATED_TIME)
            .map_err(extract_data_error)?,
        last_modified_time: row
            .try_get(properties::LAST_MODIFIED_TIME)
            .map_err(extract_data_error)?,
    });
}

fn opt_to_conditions<'a>(
    opt: &'a ExternalUserOpt,
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
    if let Some(provider_type) = opt.provider_type.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::PROVIDER_TYPE),
                operator: None,
            },
            provider_type,
        ));
    }
    if let Some(provider) = opt.provider.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::PROVIDER),
                operator: None,
            },
            provider,
        ));
    }
    if let Some(openid) = opt.openid.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::OPENID),
                operator: None,
            },
            openid,
        ));
    }
    if let Some(detail) = opt.detail.as_ref() {
        pairs.push((
            Condition {
                field: LightString::from_static(properties::DETAIL),
                operator: None,
            },
            detail,
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

pub struct ExternalUserService<'a> {
    transaction: &'a Transaction<'a>,
}

impl<'a> ExternalUserService<'a> {
    pub fn new(transaction: &'a Transaction) -> ExternalUserService<'a> {
        return ExternalUserService {
            transaction: transaction,
        };
    }

    pub async fn query_external_user_count(&self, opt: &ExternalUserOpt) -> Result<u64, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select count(1) from "{ENTITY}",\"user\""
            " where "{ENTITY}".id=\"user\".id "
            {if pairs.is_empty() {""} else {" and "}}
            {add_conditions(&mut vals, &pairs)}
        }
        .to_string();
        let statement = self
            .transaction
            .prepare(&sql)
            .await
            .map_err(prepare_statement_error)?;
        let row: Row = self
            .transaction
            .query_one(&statement, &vals)
            .await
            .map_err(query_error)?;
        let count: i64 = row.get(0);
        return Ok(count as u64);
    }

    pub async fn query_external_user(
        &self,
        page_no: u64,
        page_size: u64,
        opt: &ExternalUserOpt,
    ) -> Result<Vec<ExternalUser>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let (limit, offset) = calc_sql_pagination(page_no, page_size);
        let sql = format_xml::template! {
            "select "{ENTITY}".* from "{ENTITY}",\"user\""
            " where "{ENTITY}".id=\"user\".id "
            {if pairs.is_empty() {""} else {" and "}}
            {add_conditions(&mut vals, &pairs)}
            " limit "{limit}" offset "{offset}
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
        let list: Vec<ExternalUser> = rows
            .iter()
            .map(extract_external_user)
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }

    pub async fn query_external_user_one(
        &self,
        opt: &ExternalUserOpt,
    ) -> Result<Option<ExternalUser>, ErrNo> {
        let list = self.query_external_user(1, 1, opt).await?;
        return Ok(list.into_iter().next());
    }

    pub async fn query_external_user_batch(
        &self,
        opt: &ExternalUserOpt,
    ) -> Result<Vec<ExternalUser>, ErrNo> {
        let pairs = opt_to_conditions(opt);
        let mut vals: Vec<&(dyn ToSql + std::marker::Sync)> = Vec::with_capacity(pairs.len());
        let sql = format_xml::template! {
            "select "{ENTITY}".* from "{ENTITY}",\"user\""
            " where "{ENTITY}".id=\"user\".id "
            {if pairs.is_empty() {""} else {" and "}}
            {add_conditions(&mut vals, &pairs)}
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
        let list: Vec<ExternalUser> = rows
            .iter()
            .map(extract_external_user)
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(list);
    }
}
