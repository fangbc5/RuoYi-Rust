//! 本地缓存实现模块
//!
//! 使用 moka-rs 实现的线程安全、高性能的本地缓存

use async_trait::async_trait;
use dashmap::DashMap;
use moka::future::Cache as MokaCache;
use ruoyi_common::utils::string::{regex_from_pattern, regex_match};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::{Cache, CacheError, CacheManager, CacheResult};

/// 本地缓存配置
#[derive(Debug, Clone, Deserialize)]
pub struct LocalCacheConfig {
    /// 缓存名称
    #[serde(default = "default_name")]
    pub name: String,
    /// 最大容量
    #[serde(default = "default_max_capacity")]
    pub max_capacity: u64,
    /// 默认过期时间（秒）
    #[serde(default = "default_default_ttl")]
    pub default_ttl: u64,
    /// 缓存项过期后自动清理的时间间隔（秒）
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: u64,
}

fn default_name() -> String {
    "local".to_string()
}

fn default_max_capacity() -> u64 {
    10000
}

fn default_default_ttl() -> u64 {
    3600
}

fn default_cleanup_interval() -> u64 {
    60
}

impl Default for LocalCacheConfig {
    fn default() -> Self {
        Self {
            name: "local".to_string(),
            max_capacity: 10000,
            default_ttl: 3600,
            cleanup_interval: 60,
        }
    }
}

/// 本地缓存实现
#[derive(Clone)]
pub struct LocalCache {
    /// 普通键值缓存
    cache: Arc<MokaCache<String, Vec<u8>>>,
    /// 哈希表缓存 - 直接使用DashMap嵌套结构，避免每次访问时从moka获取并复制整个map
    hash_cache: Arc<DashMap<String, Arc<DashMap<String, Vec<u8>>>>>,
    /// 配置
    config: Arc<LocalCacheConfig>,
}

impl LocalCache {
    /// 创建新的本地缓存实例
    pub fn new(config: Arc<LocalCacheConfig>) -> Self {
        // 普通缓存
        let cache = MokaCache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.default_ttl))
            .build();

        // 哈希缓存 - 使用DashMap而不是MokaCache，以减少获取和设置操作的开销
        let hash_cache = DashMap::new();

        Self {
            cache: Arc::new(cache),
            hash_cache: Arc::new(hash_cache),
            config,
        }
    }

    pub fn get_config(&self) -> Arc<LocalCacheConfig> {
        self.config.clone()
    }

    /// 获取指定键的哈希表，如果不存在则创建
    fn get_or_create_hash(&self, key: &str) -> Arc<DashMap<String, Vec<u8>>> {
        if let Some(hash) = self.hash_cache.get(key) {
            hash.value().clone()
        } else {
            let hash = Arc::new(DashMap::new());
            self.hash_cache.insert(key.to_string(), hash.clone());
            hash
        }
    }
}

#[async_trait]
impl Cache for LocalCache {
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> CacheResult<()> {
        let serialized = serde_json::to_vec(value)?;
        self.cache.insert(key.to_string(), serialized).await;
        Ok(())
    }

    async fn set_ex<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        _ttl: Duration,
    ) -> CacheResult<()> {
        let serialized = serde_json::to_vec(value)?;
        self.cache.insert(key.to_string(), serialized).await;
        // 注意：moka在缓存项级别不支持单独设置TTL，这里我们使用的是全局配置
        // 如果需要更精细的控制，需要使用更复杂的实现
        Ok(())
    }

    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>> {
        if let Some(data) = self.cache.get(&key.to_string()).await {
            let value = serde_json::from_slice(&data)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let regex = regex_from_pattern(pattern);
        let keys: Vec<String> = self
            .cache
            .iter()
            .filter(|(k, _)| regex_match(k, &regex))
            .map(|(k, _)| k.clone().to_string())
            .collect();
        Ok(keys)
    }

    async fn del(&self, key: &str) -> CacheResult<()> {
        self.cache.invalidate(&key.to_string()).await;
        // 同时删除相关的hash结构
        self.hash_cache.remove(&key.to_string());
        Ok(())
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        Ok(self.cache.get(&key.to_string()).await.is_some() || self.hash_cache.contains_key(key))
    }

    async fn expire(&self, key: &str, _ttl: Duration) -> CacheResult<()> {
        // Moka不支持在缓存项级别设置TTL，这里只是简单地检查键是否存在
        if self.cache.get(&key.to_string()).await.is_some() || self.hash_cache.contains_key(key) {
            Ok(())
        } else {
            Err(CacheError::Other(format!("键 {} 不存在", key)))
        }
    }

    async fn incr(&self, key: &str) -> CacheResult<i64> {
        let value = if let Some(data) = self.cache.get(&key.to_string()).await {
            let current: i64 = String::from_utf8_lossy(&data)
                .parse::<i64>()
                .map_err(|e| CacheError::Deserialization(e.to_string()))?;
            current + 1
        } else {
            1
        };

        self.cache
            .insert(key.to_string(), value.to_string().into_bytes())
            .await;
        Ok(value)
    }

    async fn decr(&self, key: &str) -> CacheResult<i64> {
        let value = if let Some(data) = self.cache.get(&key.to_string()).await {
            let current: i64 = String::from_utf8_lossy(&data)
                .parse::<i64>()
                .map_err(|e| CacheError::Deserialization(e.to_string()))?;
            current - 1
        } else {
            -1
        };

        self.cache
            .insert(key.to_string(), value.to_string().into_bytes())
            .await;
        Ok(value)
    }

    // 哈希表操作的优化实现 - 直接使用DashMap而不是通过moka间接访问

    async fn hset<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> CacheResult<()> {
        let serialized = serde_json::to_vec(value)?;

        // 获取或创建哈希表
        let hash_map = self.get_or_create_hash(key);

        // 直接插入字段值，无需再次插入moka缓存
        hash_map.insert(field.to_string(), serialized);

        Ok(())
    }

    async fn hget<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
        field: &str,
    ) -> CacheResult<Option<T>> {
        // 直接从hash_cache中查找，避免获取整个哈希表的开销
        if let Some(hash_map) = self.hash_cache.get(key) {
            if let Some(data) = hash_map.get(field) {
                let value = serde_json::from_slice(data.value())?;
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    async fn hdel(&self, key: &str, field: &str) -> CacheResult<()> {
        if let Some(hash_map) = self.hash_cache.get(key) {
            hash_map.remove(field);

            // 如果哈希表为空，删除整个哈希表
            if hash_map.is_empty() {
                self.hash_cache.remove(key);
            }
        }

        Ok(())
    }

    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool> {
        if let Some(hash_map) = self.hash_cache.get(key) {
            return Ok(hash_map.contains_key(field));
        }

        Ok(false)
    }

    async fn hkeys(&self, key: &str) -> CacheResult<Vec<String>> {
        if let Some(hash_map) = self.hash_cache.get(key) {
            let mut keys = Vec::with_capacity(hash_map.len());
            for item in hash_map.iter() {
                keys.push(item.key().clone());
            }
            return Ok(keys);
        }

        Ok(Vec::new())
    }

    async fn hlen(&self, key: &str) -> CacheResult<usize> {
        if let Some(hash_map) = self.hash_cache.get(key) {
            return Ok(hash_map.len());
        }

        Ok(0)
    }

    async fn info(&self, key: Option<String>) -> CacheResult<String> {
        Ok("".to_string())
    }

    async fn dbsize(&self) -> CacheResult<usize> {
        Ok(0)
    }
}

/// 本地缓存管理器
#[derive(Clone)]
pub struct LocalCacheManager {
    config: Arc<LocalCacheConfig>,
    cache: Arc<RwLock<Option<LocalCache>>>,
}

impl LocalCacheManager {
    /// 创建新的本地缓存管理器
    pub fn new(config: Arc<LocalCacheConfig>) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// 创建使用默认配置的本地缓存管理器
    pub fn default() -> Self {
        Self::new(Arc::new(LocalCacheConfig::default()))
    }
}

#[async_trait]
impl CacheManager for LocalCacheManager {
    type CacheImpl = LocalCache;

    async fn get_cache(&self) -> CacheResult<Self::CacheImpl> {
        // 先尝试读锁获取缓存实例
        {
            let cache_guard = self.cache.read().await;
            if let Some(ref cache) = *cache_guard {
                return Ok(cache.clone());
            }
        }

        // 如果没有初始化，则使用写锁创建新实例
        let mut cache_guard = self.cache.write().await;
        if cache_guard.is_none() {
            let new_cache = LocalCache::new(self.config.clone());
            *cache_guard = Some(new_cache.clone());
            return Ok(new_cache);
        }

        // 二次检查
        Ok(cache_guard.as_ref().unwrap().clone())
    }
}
