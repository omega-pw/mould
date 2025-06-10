use serde::{Serialize, Deserialize};
use chrono::DateTime;
use chrono::Utc;
use tihu::datetime_format;
use tihu::datetime_format_opt;
use tihu::Id;
use tihu::LightString;
use tihu::Uint32;
use tihu::Uint63;
use native_common::model::Property;
use native_common::model::PropertyDefine;
use native_common::model::PropertyType;
use crate::native_common;

pub mod properties {
    pub const ID: &str = "id";
    pub const RECEIVER_TYPE: &str = "receiver_type";
    pub const RECEIVER: &str = "receiver";
    pub const SCENE: &str = "scene";
    pub const CAPTCHA: &str = "captcha";
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
    pub enum ReceiverType {
        Email = 1, //邮箱
        Phone = 2, //电话
    }
    pub fn try_i16_to_receiver_type(val: i16) -> Result<ReceiverType, LightString> {
        match val {
            1 => Ok(ReceiverType::Email),
            2 => Ok(ReceiverType::Phone),
            _ => Err(format!("未定义的接收者类型枚举值:{}", val).into())
        }
    }
    impl ToSql for ReceiverType {
        fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + 'static + Send + Sync>> {
            (*self as i16).to_sql(ty, out)
        }
        fn accepts(ty: &Type) -> bool {
            <i16 as ToSql>::accepts(ty)
        }
        to_sql_checked!();
    }
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Scene {
        Register = 1, //注册
        ResetPassword = 2, //重置密码
    }
    pub fn try_i16_to_scene(val: i16) -> Result<Scene, LightString> {
        match val {
            1 => Ok(Scene::Register),
            2 => Ok(Scene::ResetPassword),
            _ => Err(format!("未定义的场景枚举值:{}", val).into())
        }
    }
    impl ToSql for Scene {
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
 * 验证码列
 */
pub enum CaptchaProperty {
    Id(Id),
    ReceiverType(enums::ReceiverType),
    Receiver(String),
    Scene(enums::Scene),
    Captcha(String),
    CreatedTime(DateTime<Utc>),
    LastModifiedTime(DateTime<Utc>),
}

impl Property for CaptchaProperty {
    fn property_define(&self) -> PropertyDefine {
        match self {
			CaptchaProperty::Id(_) => PropertyDefine {
                key: LightString::from_static(properties::ID),
                value_type: PropertyType::Id,
				required: true,
            },
			CaptchaProperty::ReceiverType(_) => PropertyDefine {
                key: LightString::from_static(properties::RECEIVER_TYPE),
                value_type: PropertyType::Enum,
				required: true,
            },
			CaptchaProperty::Receiver(_) => PropertyDefine {
                key: LightString::from_static(properties::RECEIVER),
                value_type: PropertyType::String,
				required: true,
            },
			CaptchaProperty::Scene(_) => PropertyDefine {
                key: LightString::from_static(properties::SCENE),
                value_type: PropertyType::Enum,
				required: true,
            },
			CaptchaProperty::Captcha(_) => PropertyDefine {
                key: LightString::from_static(properties::CAPTCHA),
                value_type: PropertyType::String,
				required: true,
            },
			CaptchaProperty::CreatedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::CREATED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
			CaptchaProperty::LastModifiedTime(_) => PropertyDefine {
                key: LightString::from_static(properties::LAST_MODIFIED_TIME),
                value_type: PropertyType::DateTime,
				required: true,
            },
        }
    }
}

/**
 * 验证码
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Captcha {
    pub id: Id, //id
    pub receiver_type: enums::ReceiverType, //接收者类型
    pub receiver: String, //接收者
    pub scene: enums::Scene, //场景
    pub captcha: String, //验证码
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>, //创建时间
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>, //更新时间
}

impl Captcha {
    pub fn into_properties(self) -> Vec<CaptchaProperty> {
        return vec![
			CaptchaProperty::Id(self.id),
			CaptchaProperty::ReceiverType(self.receiver_type),
			CaptchaProperty::Receiver(self.receiver),
			CaptchaProperty::Scene(self.scene),
			CaptchaProperty::Captcha(self.captcha),
			CaptchaProperty::CreatedTime(self.created_time),
			CaptchaProperty::LastModifiedTime(self.last_modified_time),
        ];
    }
}

impl PartialEq<CaptchaProperty> for Captcha {
    fn eq(&self, property: &CaptchaProperty) -> bool {
        match property {
			CaptchaProperty::Id(id) => id == &self.id,
			CaptchaProperty::ReceiverType(receiver_type) => receiver_type == &self.receiver_type,
			CaptchaProperty::Receiver(receiver) => receiver == &self.receiver,
			CaptchaProperty::Scene(scene) => scene == &self.scene,
			CaptchaProperty::Captcha(captcha) => captcha == &self.captcha,
			CaptchaProperty::CreatedTime(created_time) => created_time == &self.created_time,
			CaptchaProperty::LastModifiedTime(last_modified_time) => last_modified_time == &self.last_modified_time,
        }
    }
}

pub struct CaptchaOpt {
    pub id: Option<Id>,
    pub receiver_type: Option<enums::ReceiverType>,
    pub receiver: Option<String>,
    pub scene: Option<enums::Scene>,
    pub captcha: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub last_modified_time: Option<DateTime<Utc>>,
}

impl CaptchaOpt {
    pub fn empty() -> CaptchaOpt {
        return CaptchaOpt {
            id: None,
            receiver_type: None,
            receiver: None,
            scene: None,
            captcha: None,
            created_time: None,
            last_modified_time: None,
        };
    }
}