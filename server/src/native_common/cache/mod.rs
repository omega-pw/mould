mod redis;

pub use self::redis::RedisCache;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cmp::Eq;
use std::fmt::Debug;
use std::hash::Hash;

/**
 * 淘汰类型
 */
pub enum EliminateType {
    Random,        //随机淘汰
    Expire(u64), //过期淘汰(单位毫秒)
    Manual,        //手动移除
}

#[async_trait]
pub trait AsyncCache {
    type Key: Hash + Eq + ?Sized;
    type Val;
    type Err;
    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Val>, Self::Err>;
    async fn set(
        &self,
        key: &Self::Key,
        value: &Self::Val,
        eliminate_type: EliminateType,
    ) -> Result<(), Self::Err>;
    async fn remove(&self, key: &Self::Key) -> Result<bool, Self::Err>;
}

#[async_trait]
pub trait QueryWithCache {
    type Client: AsyncCache<Key = [u8], Val = Vec<u8>, Err = Self::Err> + Send;
    type Key: Send + Sync + ?Sized;
    type Data: Serialize + DeserializeOwned + Send + Debug;
    type Err: Debug;
    fn cache_key(&self, key: &Self::Key) -> String;
    async fn cache_client(&self) -> Result<Self::Client, Self::Err>;
    async fn query(&self, key: &Self::Key) -> Result<Option<Self::Data>, Self::Err>;
    fn expire_time(&self) -> u64 {
        5 * 60 * 1000
    }
    async fn query_with_cache(&self, key: &Self::Key) -> Result<Option<Self::Data>, Self::Err> {
        let cache_key = self.cache_key(key);
        let cache_client = self.cache_client().await?;
        let cache = cache_client.get(cache_key.as_bytes()).await?;
        let cache_data: Option<Self::Data> = cache
            .map(|cache| match serde_json::from_slice(&cache) {
                Ok(cache) => Some(cache),
                Err(err) => {
                    log::warn!(
                        "缓存数据反序列化失败，当做没有缓存处理，key: {}, error: {:?}",
                        cache_key,
                        err
                    );
                    None
                }
            })
            .flatten();
        if let Some(cache_data) = cache_data {
            return Ok(Some(cache_data));
        } else {
            let data = self.query(key).await?;
            if let Some(data) = data {
                match serde_json::to_vec(&data) {
                    Ok(data) => {
                        if let Err(err) = cache_client
                            .set(
                                cache_key.as_bytes(),
                                &data,
                                EliminateType::Expire(self.expire_time()),
                            )
                            .await
                        {
                            log::warn!("缓存数据失败: {:?}", err);
                        }
                    }
                    Err(err) => {
                        log::warn!(
                            "数据序列化失败, key: {}, data: {:?}, error: {:?}",
                            cache_key,
                            data,
                            err
                        );
                    }
                }
                return Ok(Some(data));
            } else {
                return Ok(None);
            }
        }
    }
}
