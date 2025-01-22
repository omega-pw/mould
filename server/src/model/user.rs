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
    pub const ORG_ID: &str = "org_id";
    pub const USER_SOURCE: &str = "user_source";
    pub const NAME: &str = "name";
    pub const AVATAR_URL: &str = "avatar_url";
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
    pub enum UserSource {
        System = 1, //系统用户
        External = 2, //外部用户
    }
    pub fn try_i16_to_user_source(val: i16) -> Result<UserSource, LightString> {
        match val {
            1 => Ok(UserSource::System),
            2 => Ok(UserSource::External),
            _ => Err(format!("未定义的用户来源枚举值:{}", val).into())
        }
    }
    impl ToSql for UserSource {
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
 * 用户列
 */
pub enum UserProperty {
    Id(Id),
    OrgId(Option<Id>),
    UserSource(enums::UserSource),
    Name(String),
    AvatarUrl(Option<String>),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for UserProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			UserProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			UserProperty::OrgId(_) => PropertyDefine {
                key: LightString::from_static(properties::ORG_ID),
                value_type: PropertyType::Id,
				required: false,
            },
			UserProperty::UserSource(_) => PropertyDefine {
                key: LightString::from_static(properties::USER_SOURCE),
                value_type: PropertyType::Enum,
				required: true,
            },
			UserProperty::Name(_) => PropertyDefine {
                key: LightString::from_static(properties::NAME),
                value_type: PropertyType::String,
				required: true,
            },
			UserProperty::AvatarUrl(_) => PropertyDefine {
                key: LightString::from_static(properties::AVATAR_URL),
                value_type: PropertyType::String,
				required: false,
            },
			UserProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			UserProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 用户
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Id, //id
    pub org_id: Option<Id>, //组织id
    pub user_source: enums::UserSource, //用户来源
    pub name: String, //名称
    pub avatar_url: Option<String>, //头像
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl User {
    pub fn into_properties(self) -> Vec<UserProperty> {
        return vec![
			UserProperty::Id(self.id),
			UserProperty::OrgId(self.org_id),
			UserProperty::UserSource(self.user_source),
			UserProperty::Name(self.name),
			UserProperty::AvatarUrl(self.avatar_url),
			UserProperty::CreatedTime(self.created_time),
			UserProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<UserProperty> for User {
    fn eq(&self, property: &UserProperty) -> bool {
        match property {
			UserProperty::Id(id) => id == &self.id,
			UserProperty::OrgId(org_id) => org_id == &self.org_id,
			UserProperty::UserSource(user_source) => user_source == &self.user_source,
			UserProperty::Name(name) => name == &self.name,
			UserProperty::AvatarUrl(avatar_url) => avatar_url == &self.avatar_url,
			UserProperty::CreatedTime(created_time) => created_time == &self.created_time,
			UserProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct UserOpt {
    pub id: Option<Id>,
    pub org_id: Option<Id>,
    pub user_source: Option<enums::UserSource>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl UserOpt {
    pub fn empty() -> UserOpt {
        return UserOpt {
            id: None,
            org_id: None,
            user_source: None,
            name: None,
            avatar_url: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}