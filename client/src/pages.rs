use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Pages {
    Login,
    Index,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum SubPages {
    Welcome,
    OperatorMgr,
    // BrandMgr,
    UserMgr,
    ProductMgr,
    // RoleMgr,
    // ResourceMgr,
    // NoticeMgr,
    // FeedbackMgr,
    ChangePassword,
}
