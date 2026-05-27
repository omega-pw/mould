use crate::sdk;
use crate::utils::cache_mgr::CacheMgr;
use crate::utils::request::ApiExt;
use crate::SharedString;
use sdk::system::get_system_info::GetSystemInfoApi;
use sdk::system::get_system_info::GetSystemInfoReq;
use sdk::system::get_system_info::GetSystemInfoResp;
use std::sync::LazyLock;

pub static SYSTEM_INFO: LazyLock<CacheMgr<(), GetSystemInfoResp, SharedString>> =
    LazyLock::new(|| {
        CacheMgr::new(
            |_: ()| async move {
                let params = GetSystemInfoReq {};
                let result = GetSystemInfoApi.call(&params).await;
                result.map(move |result| {
                    let ttl = <GetSystemInfoApi as tihu::Api>::get_cache_meta(&params)
                        .map(|cache_meta| cache_meta.ttl)
                        .flatten();
                    (result, ttl)
                })
            },
            (),
        )
    });