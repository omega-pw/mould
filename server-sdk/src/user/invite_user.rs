use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::SharedString;

pub const INVITE_USER_API: &str = "/api/user/inviteUser";

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteUserReq {
    pub user_id: Id,
}

pub type InviteUserResp = ();
pub struct InviteUserApi;
impl Api for InviteUserApi {
    type Input = InviteUserReq;
    type Output = InviteUserResp;
    fn namespace() -> SharedString {
        return SharedString::from_static(INVITE_USER_API);
    }
}
