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
    pub const EMAIL: &str = "email";
    pub const USER_RANDOM_VALUE: &str = "user_random_value";
    pub const HASHED_AUTH_KEY: &str = "hashed_auth_key";
    pub const CREATED_TIME: &str = "created_time";
    pub const LAST_MODIFIED_TIME: &str = "last_modified_time";
}




/**
 * 系统用户列
 */
pub enum SystemUserProperty {
    Id(Id),
    Email(String),
    UserRandomValue(String),
    HashedAuthKey(String),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for SystemUserProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			SystemUserProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			SystemUserProperty::Email(_) => PropertyDefine {
                key: LightString::from_static(properties::EMAIL),
                value_type: PropertyType::String,
				required: true,
            },
			SystemUserProperty::UserRandomValue(_) => PropertyDefine {
                key: LightString::from_static(properties::USER_RANDOM_VALUE),
                value_type: PropertyType::String,
				required: true,
            },
			SystemUserProperty::HashedAuthKey(_) => PropertyDefine {
                key: LightString::from_static(properties::HASHED_AUTH_KEY),
                value_type: PropertyType::String,
				required: true,
            },
			SystemUserProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			SystemUserProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 系统用户
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct SystemUser {
    pub id: Id, //用户id
    pub email: String, //邮箱
    pub user_random_value: String, //随机数
    pub hashed_auth_key: String, //授权秘钥摘要
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl SystemUser {
    pub fn into_properties(self) -> Vec<SystemUserProperty> {
        return vec![
			SystemUserProperty::Id(self.id),
			SystemUserProperty::Email(self.email),
			SystemUserProperty::UserRandomValue(self.user_random_value),
			SystemUserProperty::HashedAuthKey(self.hashed_auth_key),
			SystemUserProperty::CreatedTime(self.created_time),
			SystemUserProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<SystemUserProperty> for SystemUser {
    fn eq(&self, property: &SystemUserProperty) -> bool {
        match property {
			SystemUserProperty::Id(id) => id == &self.id,
			SystemUserProperty::Email(email) => email == &self.email,
			SystemUserProperty::UserRandomValue(user_random_value) => user_random_value == &self.user_random_value,
			SystemUserProperty::HashedAuthKey(hashed_auth_key) => hashed_auth_key == &self.hashed_auth_key,
			SystemUserProperty::CreatedTime(created_time) => created_time == &self.created_time,
			SystemUserProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct SystemUserOpt {
    pub id: Option<Id>,
    pub email: Option<String>,
    pub user_random_value: Option<String>,
    pub hashed_auth_key: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl SystemUserOpt {
    pub fn empty() -> SystemUserOpt {
        return SystemUserOpt {
            id: None,
            email: None,
            user_random_value: None,
            hashed_auth_key: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}