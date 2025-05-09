// ruoyi-framework/src/config/redis.rs
//! Redis 配置模块

use std::sync::Arc;

use serde::Deserialize;

use crate::cache::{LocalCacheConfig, MultiLevelCacheConfig, RedisConfig};

/// 缓存类型枚举
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    /// Redis缓存
    Redis,
    /// 本地缓存
    Local,
    /// 多级缓存 (Redis + 本地)
    Multi,
}

impl Default for CacheType {
    fn default() -> Self {
        CacheType::Local
    }
}

/// 缓存配置详情
#[derive(Debug, Deserialize, Clone)]
pub struct CacheSettings {
    /// 是否启用缓存
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// 缓存类型
    #[serde(default)]
    pub cache_type: CacheType,

    /// Redis缓存配置
    #[serde(default)]
    pub redis: Arc<RedisConfig>,

    /// 本地缓存配置
    #[serde(default)]
    pub local: Arc<LocalCacheConfig>,

    /// 多级缓存配置
    #[serde(default)]
    pub multi: Arc<MultiLevelCacheConfig>,
}

fn default_enabled() -> bool {
    true
}
