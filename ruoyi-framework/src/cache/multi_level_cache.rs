//! 多级缓存实现模块
//!
//! 结合本地缓存和Redis缓存的优势，提供高性能的缓存解决方案
//! 两级缓存策略:
//! 1. 读操作：优先从本地缓存读取，如果不存在则从Redis读取并回填本地缓存
//! 2. 写操作：同时写入本地缓存和Redis缓存，确保一致性
//! 3. 删除操作：同时在本地缓存和Redis缓存中删除
//! 4. 过期操作：设置两级缓存的过期时间

use async_trait::async_trait;
use log::{debug, error, info, warn};
use ruoyi_common::utils::string::regex_from_pattern;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::config::cache::CacheSettings;

use super::{
    Cache, CacheManager, CacheResult, LocalCache, LocalCacheManager, RedisCache, RedisCacheManager,
};

/// 多级缓存配置
#[derive(Debug, Clone, Deserialize)]
pub struct MultiLevelCacheConfig {
    /// 本地缓存过期时间（秒）- 通常比Redis设置更短以确保数据最终一致性
    #[serde(default = "default_local_ttl")]
    pub local_ttl: u64,
    /// 是否在Redis连接失败时使用只读本地缓存模式
    #[serde(default = "default_fallback_to_local")]
    pub fallback_to_local: bool,
}

fn default_local_ttl() -> u64 {
    300
}

fn default_fallback_to_local() -> bool {
    true
}

impl Default for MultiLevelCacheConfig {
    fn default() -> Self {
        Self {
            local_ttl: 300, // 默认5分钟
            fallback_to_local: true,
        }
    }
}

/// 多级缓存实现
#[derive(Clone)]
pub struct MultiLevelCache {
    /// 本地缓存
    local_cache: Arc<LocalCache>,
    /// Redis缓存
    redis_cache: Option<Arc<RedisCache>>,
    /// 配置
    config: Arc<MultiLevelCacheConfig>,
    /// 是否已降级到只读本地缓存模式
    is_fallback_mode: bool,
}

impl MultiLevelCache {
    /// 创建新的多级缓存实例
    pub async fn new(config: Arc<CacheSettings>) -> CacheResult<Self> {
        // 初始化本地缓存
        let local_cache_manager = LocalCacheManager::new(config.local.clone());
        let local_cache = Arc::new(local_cache_manager.get_cache().await?);

        // 尝试初始化Redis缓存
        let redis_result = RedisCacheManager::new(config.redis.clone())
            .get_cache()
            .await;
        let (redis_cache, is_fallback) = match redis_result {
            Ok(redis) => {
                info!("多级缓存：已成功连接Redis服务器");
                (Some(Arc::new(redis)), false)
            }
            Err(e) => {
                if config.multi.fallback_to_local {
                    warn!("多级缓存：无法连接Redis服务器，已降级为本地缓存模式: {}", e);
                    (None, true)
                } else {
                    error!("多级缓存：无法连接Redis服务器，且未配置降级策略: {}", e);
                    return Err(e);
                }
            }
        };

        Ok(Self {
            local_cache,
            redis_cache,
            config: config.multi.clone(),
            is_fallback_mode: is_fallback,
        })
    }

    /// 检查是否处于降级模式
    pub fn is_in_fallback_mode(&self) -> bool {
        self.is_fallback_mode
    }

    /// 获取本地缓存TTL
    fn get_local_ttl(&self) -> Duration {
        Duration::from_secs(self.config.local_ttl)
    }
}

#[async_trait]
impl Cache for MultiLevelCache {
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> CacheResult<()> {
        // 先设置本地缓存（使用配置的本地TTL）
        let _serialized_value = serde_json::to_vec(value)?;
        self.local_cache
            .set_ex(key, value, self.get_local_ttl())
            .await?;

        // 如果Redis可用，也设置Redis缓存
        if let Some(redis) = &self.redis_cache {
            match redis.set(key, value).await {
                Ok(_) => debug!("多级缓存：键 {} 已成功写入Redis", key),
                Err(e) => {
                    warn!("多级缓存：键 {} 写入Redis失败: {}", key, e);
                    // 这里我们继续执行，因为至少本地缓存已成功设置
                }
            }
        }

        Ok(())
    }

    async fn set_ex<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<()> {
        // 对本地缓存，使用配置的本地TTL和提供的TTL中的较小值
        let local_ttl = std::cmp::min(ttl, self.get_local_ttl());
        self.local_cache.set_ex(key, value, local_ttl).await?;

        // 如果Redis可用，使用提供的TTL
        if let Some(redis) = &self.redis_cache {
            match redis.set_ex(key, value, ttl).await {
                Ok(_) => debug!("多级缓存：键 {} 已成功写入Redis，TTL: {:?}", key, ttl),
                Err(e) => warn!("多级缓存：键 {} 写入Redis失败: {}", key, e),
            }
        }

        Ok(())
    }

    async fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        // 首先尝试从本地缓存获取
        match self.local_cache.get::<T>(key).await {
            Ok(Some(value)) => {
                debug!("多级缓存：键 {} 从本地缓存命中", key);
                return Ok(Some(value));
            }
            Ok(None) => debug!("多级缓存：键 {} 在本地缓存中不存在", key),
            Err(e) => warn!("多级缓存：从本地缓存获取键 {} 失败: {}", key, e),
        }

        // 如果本地缓存未命中且Redis可用，尝试从Redis获取
        if let Some(redis) = &self.redis_cache {
            match redis.get::<T>(key).await {
                Ok(Some(value)) => {
                    debug!("多级缓存：键 {} 从Redis命中", key);
                    // 不尝试回填本地缓存，因为T可能没有实现Serialize
                    // 在需要回填的场景，用户应该使用实现了Serialize的类型
                    return Ok(Some(value));
                }
                Ok(None) => debug!("多级缓存：键 {} 在Redis中也不存在", key),
                Err(e) => warn!("多级缓存：从Redis获取键 {} 失败: {}", key, e),
            }
        }

        // 如果两级缓存都未命中，则返回None
        Ok(None)
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let mut keys: Vec<String> = self.local_cache.keys(pattern).await?;
        if !keys.is_empty() {
            return Ok(keys);
        }
        if let Some(redis) = &self.redis_cache {
            let redis_keys = redis.keys(pattern).await?;
            keys.extend(redis_keys);
        }
        Ok(keys)
    }

    async fn del(&self, key: &str) -> CacheResult<()> {
        // 从本地缓存删除
        if let Err(e) = self.local_cache.del(key).await {
            warn!("多级缓存：从本地缓存删除键 {} 失败: {}", key, e);
        }

        // 如果Redis可用，也从Redis删除
        if let Some(redis) = &self.redis_cache {
            if let Err(e) = redis.del(key).await {
                warn!("多级缓存：从Redis删除键 {} 失败: {}", key, e);
            }
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        // 先检查本地缓存
        match self.local_cache.exists(key).await {
            Ok(true) => return Ok(true),
            Ok(false) => debug!("多级缓存：键 {} 在本地缓存中不存在", key),
            Err(e) => warn!("多级缓存：检查本地缓存键 {} 存在性失败: {}", key, e),
        }

        // 如果本地不存在且Redis可用，检查Redis
        if let Some(redis) = &self.redis_cache {
            match redis.exists(key).await {
                Ok(true) => {
                    debug!("多级缓存：键 {} 在Redis中存在", key);
                    return Ok(true);
                }
                Ok(false) => return Ok(false),
                Err(e) => warn!("多级缓存：检查Redis键 {} 存在性失败: {}", key, e),
            }
        }

        // 如果都检查失败，假定不存在
        Ok(false)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        // 对本地缓存，使用配置的本地TTL和提供的TTL中的较小值
        let local_ttl = std::cmp::min(ttl, self.get_local_ttl());
        if let Err(e) = self.local_cache.expire(key, local_ttl).await {
            warn!("多级缓存：设置本地缓存键 {} 的过期时间失败: {}", key, e);
        }

        // 如果Redis可用，使用提供的TTL
        if let Some(redis) = &self.redis_cache {
            if let Err(e) = redis.expire(key, ttl).await {
                warn!("多级缓存：设置Redis键 {} 的过期时间失败: {}", key, e);
            }
        }

        Ok(())
    }

    async fn incr(&self, key: &str) -> CacheResult<i64> {
        // 如果Redis可用，优先在Redis中递增（保证计数器准确性）
        if let Some(redis) = &self.redis_cache {
            match redis.incr(key).await {
                Ok(value) => {
                    // 更新本地缓存
                    if let Err(e) = self
                        .local_cache
                        .set_ex(key, &value.to_string(), self.get_local_ttl())
                        .await
                    {
                        warn!("多级缓存：更新本地缓存键 {} 的递增值失败: {}", key, e);
                    }
                    return Ok(value);
                }
                Err(e) => {
                    warn!("多级缓存：在Redis中递增键 {} 失败: {}", key, e);
                    // Redis操作失败，使用本地缓存作为降级方案
                }
            }
        }

        // Redis不可用或操作失败，使用本地缓存
        self.local_cache.incr(key).await
    }

    async fn decr(&self, key: &str) -> CacheResult<i64> {
        // 如果Redis可用，优先在Redis中递减
        if let Some(redis) = &self.redis_cache {
            match redis.decr(key).await {
                Ok(value) => {
                    // 更新本地缓存
                    if let Err(e) = self
                        .local_cache
                        .set_ex(key, &value.to_string(), self.get_local_ttl())
                        .await
                    {
                        warn!("多级缓存：更新本地缓存键 {} 的递减值失败: {}", key, e);
                    }
                    return Ok(value);
                }
                Err(e) => {
                    warn!("多级缓存：在Redis中递减键 {} 失败: {}", key, e);
                    // Redis操作失败，使用本地缓存作为降级方案
                }
            }
        }

        // Redis不可用或操作失败，使用本地缓存
        self.local_cache.decr(key).await
    }

    // 哈希表操作的实现

    async fn hset<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> CacheResult<()> {
        // 先设置本地缓存
        self.local_cache.hset(key, field, value).await?;

        // 如果Redis可用，也设置Redis缓存
        if let Some(redis) = &self.redis_cache {
            if let Err(e) = redis.hset(key, field, value).await {
                warn!(
                    "多级缓存：在Redis中设置哈希表键 {}::{} 失败: {}",
                    key, field, e
                );
            }
        }

        Ok(())
    }

    async fn hget<T>(&self, key: &str, field: &str) -> CacheResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        // 首先尝试从本地缓存获取
        match self.local_cache.hget::<T>(key, field).await {
            Ok(Some(value)) => {
                debug!("多级缓存：哈希表键 {}::{} 从本地缓存命中", key, field);
                return Ok(Some(value));
            }
            Ok(None) => debug!("多级缓存：哈希表键 {}::{} 在本地缓存中不存在", key, field),
            Err(e) => warn!(
                "多级缓存：从本地缓存获取哈希表键 {}::{} 失败: {}",
                key, field, e
            ),
        }

        // 如果本地缓存未命中且Redis可用，尝试从Redis获取
        if let Some(redis) = &self.redis_cache {
            match redis.hget::<T>(key, field).await {
                Ok(Some(value)) => {
                    debug!("多级缓存：哈希表键 {}::{} 从Redis命中", key, field);
                    // 不尝试回填本地缓存，因为T可能没有实现Serialize
                    return Ok(Some(value));
                }
                Ok(None) => debug!("多级缓存：哈希表键 {}::{} 在Redis中也不存在", key, field),
                Err(e) => warn!(
                    "多级缓存：从Redis获取哈希表键 {}::{} 失败: {}",
                    key, field, e
                ),
            }
        }

        // 如果两级缓存都未命中，则返回None
        Ok(None)
    }

    async fn hdel(&self, key: &str, field: &str) -> CacheResult<()> {
        // 从本地缓存删除
        if let Err(e) = self.local_cache.hdel(key, field).await {
            warn!(
                "多级缓存：从本地缓存删除哈希表键 {}::{} 失败: {}",
                key, field, e
            );
        }

        // 如果Redis可用，也从Redis删除
        if let Some(redis) = &self.redis_cache {
            if let Err(e) = redis.hdel(key, field).await {
                warn!(
                    "多级缓存：从Redis删除哈希表键 {}::{} 失败: {}",
                    key, field, e
                );
            }
        }

        Ok(())
    }

    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool> {
        // 先检查本地缓存
        match self.local_cache.hexists(key, field).await {
            Ok(true) => return Ok(true),
            Ok(false) => debug!("多级缓存：哈希表键 {}::{} 在本地缓存中不存在", key, field),
            Err(e) => warn!(
                "多级缓存：检查本地缓存哈希表键 {}::{} 存在性失败: {}",
                key, field, e
            ),
        }

        // 如果本地不存在且Redis可用，检查Redis
        if let Some(redis) = &self.redis_cache {
            match redis.hexists(key, field).await {
                Ok(true) => {
                    debug!("多级缓存：哈希表键 {}::{} 在Redis中存在", key, field);
                    return Ok(true);
                }
                Ok(false) => return Ok(false),
                Err(e) => warn!(
                    "多级缓存：检查Redis哈希表键 {}::{} 存在性失败: {}",
                    key, field, e
                ),
            }
        }

        // 如果都检查失败，假定不存在
        Ok(false)
    }

    async fn hkeys(&self, key: &str) -> CacheResult<Vec<String>> {
        let mut result = Vec::new();
        let mut local_success = false;

        // 首先尝试从本地缓存获取
        match self.local_cache.hkeys(key).await {
            Ok(keys) => {
                if !keys.is_empty() {
                    local_success = true;
                    result = keys;
                }
            }
            Err(e) => warn!("多级缓存：从本地缓存获取哈希表 {} 的所有键失败: {}", key, e),
        }

        // 如果本地缓存失败或为空且Redis可用，尝试从Redis获取
        if !local_success && self.redis_cache.is_some() {
            if let Some(redis) = &self.redis_cache {
                match redis.hkeys(key).await {
                    Ok(keys) => {
                        result = keys;
                        // 可以考虑回填本地缓存，但可能需要逐个字段获取值
                    }
                    Err(e) => warn!("多级缓存：从Redis获取哈希表 {} 的所有键失败: {}", key, e),
                }
            }
        }

        Ok(result)
    }

    async fn hlen(&self, key: &str) -> CacheResult<usize> {
        let mut local_len = 0;
        let mut local_success = false;

        // 首先尝试从本地缓存获取
        match self.local_cache.hlen(key).await {
            Ok(len) => {
                if len > 0 {
                    local_success = true;
                    local_len = len;
                }
            }
            Err(e) => warn!("多级缓存：从本地缓存获取哈希表 {} 的长度失败: {}", key, e),
        }

        // 如果本地缓存失败或为空且Redis可用，尝试从Redis获取
        if !local_success && self.redis_cache.is_some() {
            if let Some(redis) = &self.redis_cache {
                match redis.hlen(key).await {
                    Ok(len) => return Ok(len),
                    Err(e) => warn!("多级缓存：从Redis获取哈希表 {} 的长度失败: {}", key, e),
                }
            }
        }

        Ok(local_len)
    }

    async fn info(&self, key: Option<String>) -> CacheResult<String> {
        if let Some(redis) = &self.redis_cache {
            redis.info(key).await
        } else {
            Ok("".to_string())
        }
    }

    async fn dbsize(&self) -> CacheResult<usize> {
        if let Some(redis) = &self.redis_cache {
            redis.dbsize().await
        } else {
            Ok(0)
        }
    }
}
/// 多级缓存管理器
pub struct MultiLevelCacheManager {
    config: Arc<CacheSettings>,
    cache: Option<MultiLevelCache>,
}

impl MultiLevelCacheManager {
    /// 创建新的多级缓存管理器
    pub fn new(config: Arc<CacheSettings>) -> Self {
        Self {
            config,
            cache: None,
        }
    }
}

#[async_trait]
impl CacheManager for MultiLevelCacheManager {
    type CacheImpl = MultiLevelCache;

    async fn get_cache(&self) -> CacheResult<Self::CacheImpl> {
        match &self.cache {
            Some(cache) => Ok(cache.clone()),
            None => {
                let cache = MultiLevelCache::new(self.config.clone()).await?;
                Ok(cache)
            }
        }
    }
}
