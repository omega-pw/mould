use crate::sdk;
use crate::utils::cache_mgr::CacheMgr;
use crate::utils::request::ApiExt;
use crate::SharedString;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyApi;
use sdk::auth::get_rsa_pub_key::GetRsaPubKeyReq;
use sdk::system::get_turnstile_site_key::GetTurnstileSiteKeyApi;
use sdk::system::get_turnstile_site_key::GetTurnstileSiteKeyReq;
use std::sync::LazyLock;

pub static RSA_PUB_KEY: LazyLock<CacheMgr<(), SharedString, SharedString>> = LazyLock::new(|| {
    CacheMgr::new(
        |_: ()| async move {
            let params = GetRsaPubKeyReq {};
            let result = GetRsaPubKeyApi.call(&params).await;
            result.map(move |result| {
                let ttl = <GetRsaPubKeyApi as tihu::Api>::get_cache_meta(&params)
                    .map(|cache_meta| cache_meta.ttl)
                    .flatten();
                (SharedString::from(result), ttl)
            })
        },
        (),
    )
});

pub static TURNSTILE_SITE_KEY: LazyLock<CacheMgr<(), Option<SharedString>, SharedString>> =
    LazyLock::new(|| {
        CacheMgr::new(
            |_: ()| async move {
                let params = GetTurnstileSiteKeyReq {};
                let result = GetTurnstileSiteKeyApi.call(&params).await;
                result.map(move |result| {
                    let ttl = <GetTurnstileSiteKeyApi as tihu::Api>::get_cache_meta(&params)
                        .map(|cache_meta| cache_meta.ttl)
                        .flatten();
                    (result.site_key.map(SharedString::from), ttl)
                })
            },
            (),
        )
    });
