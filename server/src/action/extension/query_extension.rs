use crate::prehandle::auth::User;
use crate::sdk;
use crate::Context;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::query_extension::QueryExtensionResp;
use std::sync::Arc;
use tihu_native::ErrNo;

pub async fn query_extension(
    context: Arc<Context>,
    _user: User,
    _query_extension_req: QueryExtensionReq,
) -> Result<QueryExtensionResp, ErrNo> {
    let extensions = context.get_extensions();
    return Ok(extensions);
}
