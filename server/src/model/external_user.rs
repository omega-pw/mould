use serde::{Serialize, Deserialize};
use chrono::DateTime;
use chrono::Utc;
use tihu::datetime_format;
use tihu::datetime_format_opt;
use tihu::Id;
use tihu::LightString;
use native_common::model::Property;
use native_common::model::PropertyDefine;
use native_common::model::PropertyType;
use crate::native_common;

pub mod properties {
    pub const ID: &str = "id";
    pub const PROVIDER_TYPE: &str = "provider_type";
    pub const PROVIDER: &str = "provider";
    pub const OPENID: &str = "openid";
    pub const DETAIL: &str = "detail";
    pub const CREATED_TIME: &str = "created_time";
    pub const LAST_MODIFIED_TIME: &str = "last_modified_time";
}

pub mod enums {
    use tihu::LightString;
    use std::error::Error;
    use serde::{Serialize, Deserialize};
    use tokio_postgres::types::{ToSql, Type, IsNull, to_sql_checked};
    use bytes::BytesMut;
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum ProviderType {
        Openid = 1, //Open Id
        Oauth2 = 2, //Oauth2
    }
    pub fn try_i16_to_provider_type(val: i16) -> Result<ProviderType, LightString> {
        match val {
            1 => Ok(ProviderType::Openid),
            2 => Ok(ProviderType::Oauth2),
            _ => Err(format!("未定义的提供者类型枚举值:{}", val).into())
        }
    }
    impl ToSql for ProviderType {
        fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + 'static + Send + Sync>> {
            (*self as i16).to_sql(ty, out)
        }
        fn accepts(ty: &Type) -> bool {
            <i16 as ToSql>::accepts(ty)
        }
        to_sql_checked!();
    }
}


/**
 * 外部用户列
 */
pub enum ExternalUserProperty {
    Id(Id),
    ProviderType(enums::ProviderType),
    Provider(String),
    Openid(String),
    Detail(Option<String>),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for ExternalUserProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			ExternalUserProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			ExternalUserProperty::ProviderType(_) => PropertyDefine {
                key: LightString::from_static(properties::PROVIDER_TYPE),
                value_type: PropertyType::Enum,
				required: true,
            },
			ExternalUserProperty::Provider(_) => PropertyDefine {
                key: LightString::from_static(properties::PROVIDER),
                value_type: PropertyType::String,
				required: true,
            },
			ExternalUserProperty::Openid(_) => PropertyDefine {
                key: LightString::from_static(properties::OPENID),
                value_type: PropertyType::String,
				required: true,
            },
			ExternalUserProperty::Detail(_) => PropertyDefine {
                key: LightString::from_static(properties::DETAIL),
                value_type: PropertyType::String,
				required: false,
            },
			ExternalUserProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			ExternalUserProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 外部用户
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalUser {
    pub id: Id, //用户id
    pub provider_type: enums::ProviderType, //提供者类型
    pub provider: String, //提供者
    pub openid: String, //开放id
    pub detail: Option<String>, //详细信息
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl ExternalUser {
    pub fn into_properties(self) -> Vec<ExternalUserProperty> {
        return vec![
			ExternalUserProperty::Id(self.id),
			ExternalUserProperty::ProviderType(self.provider_type),
			ExternalUserProperty::Provider(self.provider),
			ExternalUserProperty::Openid(self.openid),
			ExternalUserProperty::Detail(self.detail),
			ExternalUserProperty::CreatedTime(self.created_time),
			ExternalUserProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<ExternalUserProperty> for ExternalUser {
    fn eq(&self, property: &ExternalUserProperty) -> bool {
        match property {
			ExternalUserProperty::Id(id) => id == &self.id,
			ExternalUserProperty::ProviderType(provider_type) => provider_type == &self.provider_type,
			ExternalUserProperty::Provider(provider) => provider == &self.provider,
			ExternalUserProperty::Openid(openid) => openid == &self.openid,
			ExternalUserProperty::Detail(detail) => detail == &self.detail,
			ExternalUserProperty::CreatedTime(created_time) => created_time == &self.created_time,
			ExternalUserProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct ExternalUserOpt {
    pub id: Option<Id>,
    pub provider_type: Option<enums::ProviderType>,
    pub provider: Option<String>,
    pub openid: Option<String>,
    pub detail: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl ExternalUserOpt {
    pub fn empty() -> ExternalUserOpt {
        return ExternalUserOpt {
            id: None,
            provider_type: None,
            provider: None,
            openid: None,
            detail: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}