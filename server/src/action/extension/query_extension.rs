use crate::get_context;
use crate::middleware::auth::User;
use crate::sdk;
use sdk::extension::query_extension::QueryExtensionReq;
use sdk::extension::query_extension::QueryExtensionResp;
use tihu::Id;
use tihu_native::ErrNo;

pub async fn query_extension(
    _org_id: Id,
    _user: User,
    _query_extension_req: QueryExtensionReq,
) -> Result<QueryExtensionResp, ErrNo> {
    let context = get_context()?;
    let extensions = context.get_extensions();
    return Ok(extensions);
}
