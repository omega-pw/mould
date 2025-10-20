use crate::SharedString;

pub enum PropertyType {
    Enum,
    SmallInt,
    Bool,
    Integer,
    Id,
    Float,
    String,
    Binary,
    DateTime,
}

pub struct PropertyDefine {
    pub key: SharedString,
    pub value_type: PropertyType,
    pub required: bool,
}

pub trait Property {
    fn property_define(&self) -> PropertyDefine;
}
