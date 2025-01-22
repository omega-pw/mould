use super::AsyncCache;
use super::EliminateType;
use async_trait::async_trait;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Connection;
use deadpool_redis::Pool;
use std::sync::Arc;
use tihu_native::ErrNo;

#[derive(Clone)]
pub struct RedisCache {
    redis_pool: Option<Arc<Pool>>,
}

impl RedisCache {
    pub fn new(redis_pool: Option<Arc<Pool>>) -> RedisCache {
        RedisCache {
            redis_pool: redis_pool,
        }
    }
    async fn get_client(&self) -> Result<Connection, ErrNo> {
        let pool = self.redis_pool.as_ref().ok_or(ErrNo::NoCacheClient)?;
        pool.get().await.map_err(|err| ErrNo::Other(err.into()))
    }
}

#[async_trait]
impl AsyncCache for RedisCache {
    type Key = [u8];
    type Val = Vec<u8>;
    type Err = ErrNo;
    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Val>, Self::Err> {
        let mut cache_client = self.get_client().await?;
        let val: Option<Vec<u8>> = cache_client.get(key).await.map_err(|err| -> ErrNo {
            log::error!("获取缓存失败: {:?}", err);
            ErrNo::CacheOperationError(err.into())
        })?;
        Ok(val)
    }
    async fn set(
        &self,
        key: &Self::Key,
        value: &Self::Val,
        eliminate_type: EliminateType,
    ) -> Result<(), Self::Err> {
        let mut cache_client = self.get_client().await?;
        match eliminate_type {
            EliminateType::Expire(duration) => cache_client
                .pset_ex(key, value.as_slice(), duration)
                .await
                .map_err(|err| -> ErrNo {
                    log::error!("保存缓存失败: {:?}", err);
                    ErrNo::CacheOperationError(err.into())
                }),
            _ => cache_client
                .set(key, value.as_slice())
                .await
                .map_err(|err| -> ErrNo {
                    log::error!("保存缓存失败: {:?}", err);
                    ErrNo::CacheOperationError(err.into())
                }),
        }
    }
    async fn remove(&self, key: &Self::Key) -> Result<bool, Self::Err> {
        let mut cache_client = self.get_client().await?;
        return cache_client.del(key).await.map_err(|err| -> ErrNo {
            log::error!("删除缓存失败: {:?}", err);
            ErrNo::CacheOperationError(err.into())
        });
    }
}
