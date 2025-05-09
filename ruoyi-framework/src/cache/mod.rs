//! 缓存管理模块
//!
//! 提供统一的缓存访问接口，支持本地缓存和分布式缓存

mod error;
mod examples;
pub mod global_cache;
mod local_cache;
mod multi_level_cache;
mod redis_cache;

pub use error::{CacheError, CacheResult};
pub use examples::{
    cache_admin_example, global_cache_example, hash_performance_test, local_cache_example,
};
#[cfg(test)]
pub use global_cache::reset_global_cache;
pub use global_cache::{
    get_global_cache, init_global_cache, init_global_cache_async, is_global_cache_initialized,
};
pub use local_cache::{LocalCache, LocalCacheConfig, LocalCacheManager};
pub use multi_level_cache::{MultiLevelCache, MultiLevelCacheConfig, MultiLevelCacheManager};
pub use redis_cache::{RedisCache, RedisCacheManager, RedisConfig, RedisConnectionType};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// 对象安全的缓存接口，专用于字符串和基本类型操作
#[async_trait]
pub trait CacheBase: Send + Sync + 'static {
    /// 设置字符串缓存
    async fn set_string(&self, key: &str, value: &str) -> CacheResult<()>;

    /// 设置带过期时间的字符串缓存
    async fn set_string_ex(&self, key: &str, value: &str, ttl: Duration) -> CacheResult<()>;

    /// 获取字符串缓存
    async fn get_string(&self, key: &str) -> CacheResult<Option<String>>;

    /// 设置整数缓存
    async fn set_int(&self, key: &str, value: i64) -> CacheResult<()>;

    /// 获取整数缓存
    async fn get_int(&self, key: &str) -> CacheResult<Option<i64>>;

    /// 获取缓存key集合 
    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>>;

    /// 删除缓存
    async fn del(&self, key: &str) -> CacheResult<()>;

    /// 判断键是否存在
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// 设置过期时间
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()>;

    /// 递增操作
    async fn incr(&self, key: &str) -> CacheResult<i64>;

    /// 递减操作
    async fn decr(&self, key: &str) -> CacheResult<i64>;

    /// 设置哈希表字符串字段值
    async fn hset_string(&self, key: &str, field: &str, value: &str) -> CacheResult<()>;

    /// 获取哈希表字符串字段值
    async fn hget_string(&self, key: &str, field: &str) -> CacheResult<Option<String>>;

    /// 设置哈希表整数字段值
    async fn hset_int(&self, key: &str, field: &str, value: i64) -> CacheResult<()>;

    /// 获取哈希表整数字段值
    async fn hget_int(&self, key: &str, field: &str) -> CacheResult<Option<i64>>;

    /// 删除哈希表字段
    async fn hdel(&self, key: &str, field: &str) -> CacheResult<()>;

    /// 判断哈希表字段是否存在
    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool>;

    /// 获取哈希表所有字段
    async fn hkeys(&self, key: &str) -> CacheResult<Vec<String>>;

    /// 获取哈希表中字段数量
    async fn hlen(&self, key: &str) -> CacheResult<usize>;

    /// 获取Redis信息
    async fn info(&self, key: Option<String>) -> CacheResult<String>;

    /// 获取Redis数据库大小
    async fn dbsize(&self) -> CacheResult<usize>;
}

/// 通用缓存接口
#[async_trait]
pub trait Cache: Send + Sync + 'static {
    /// 设置缓存
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> CacheResult<()>;

    /// 设置带过期时间的缓存
    async fn set_ex<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<()>;

    /// 获取缓存
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>>;

    /// 获取缓存key集合 
    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>>;

    /// 删除缓存
    async fn del(&self, key: &str) -> CacheResult<()>;

    /// 判断键是否存在
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// 设置过期时间
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()>;

    /// 递增操作
    async fn incr(&self, key: &str) -> CacheResult<i64>;

    /// 递减操作
    async fn decr(&self, key: &str) -> CacheResult<i64>;

    /// 设置哈希表字段值
    async fn hset<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> CacheResult<()>;

    /// 获取哈希表字段值
    async fn hget<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
        field: &str,
    ) -> CacheResult<Option<T>>;

    /// 删除哈希表字段
    async fn hdel(&self, key: &str, field: &str) -> CacheResult<()>;

    /// 判断哈希表字段是否存在
    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool>;

    /// 获取哈希表所有字段
    async fn hkeys(&self, key: &str) -> CacheResult<Vec<String>>;

    /// 获取哈希表中字段数量
    async fn hlen(&self, key: &str) -> CacheResult<usize>;

    /// 获取Redis信息
    async fn info(&self, key: Option<String>) -> CacheResult<String>;

    /// 获取Redis数据库大小
    async fn dbsize(&self) -> CacheResult<usize>;
}

/// 缓存管理器接口
#[async_trait]
pub trait CacheManager: Send + Sync + 'static {
    /// 缓存实现类型
    type CacheImpl: Cache;

    /// 获取缓存实例
    async fn get_cache(&self) -> CacheResult<Self::CacheImpl>;
}

/// 缓存适配器，将Cache实现包装为CacheBase
pub struct CacheAdapter<T: Cache> {
    inner: T,
}

impl<T: Cache> CacheAdapter<T> {
    /// 创建新的缓存适配器
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<T: Cache> CacheBase for CacheAdapter<T> {
    async fn set_string(&self, key: &str, value: &str) -> CacheResult<()> {
        self.inner.set(key, &value.to_string()).await
    }

    async fn set_string_ex(&self, key: &str, value: &str, ttl: Duration) -> CacheResult<()> {
        self.inner.set_ex(key, &value.to_string(), ttl).await
    }

    async fn get_string(&self, key: &str) -> CacheResult<Option<String>> {
        self.inner.get(key).await
    }

    async fn set_int(&self, key: &str, value: i64) -> CacheResult<()> {
        self.inner.set(key, &value).await
    }

    async fn get_int(&self, key: &str) -> CacheResult<Option<i64>> {
        self.inner.get(key).await
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        self.inner.keys(pattern).await
    }

    async fn del(&self, key: &str) -> CacheResult<()> {
        self.inner.del(key).await
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        self.inner.exists(key).await
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        self.inner.expire(key, ttl).await
    }

    async fn incr(&self, key: &str) -> CacheResult<i64> {
        self.inner.incr(key).await
    }

    async fn decr(&self, key: &str) -> CacheResult<i64> {
        self.inner.decr(key).await
    }

    async fn hset_string(&self, key: &str, field: &str, value: &str) -> CacheResult<()> {
        self.inner.hset(key, field, &value.to_string()).await
    }

    async fn hget_string(&self, key: &str, field: &str) -> CacheResult<Option<String>> {
        self.inner.hget(key, field).await
    }

    async fn hset_int(&self, key: &str, field: &str, value: i64) -> CacheResult<()> {
        self.inner.hset(key, field, &value).await
    }

    async fn hget_int(&self, key: &str, field: &str) -> CacheResult<Option<i64>> {
        self.inner.hget(key, field).await
    }

    async fn hdel(&self, key: &str, field: &str) -> CacheResult<()> {
        self.inner.hdel(key, field).await
    }

    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool> {
        self.inner.hexists(key, field).await
    }

    async fn hkeys(&self, key: &str) -> CacheResult<Vec<String>> {
        self.inner.hkeys(key).await
    }

    async fn hlen(&self, key: &str) -> CacheResult<usize> {
        self.inner.hlen(key).await
    }

    async fn info(&self, key: Option<String>) -> CacheResult<String> {
        self.inner.info(key).await
    }

    async fn dbsize(&self) -> CacheResult<usize> {
        self.inner.dbsize().await
    }
}
