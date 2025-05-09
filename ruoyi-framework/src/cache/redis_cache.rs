//! Redis缓存实现模块
//!
//! 支持单机Redis和Redis集群模式

use async_trait::async_trait;
use log::info;
use redis::{
    aio::ConnectionManager, cluster::ClusterClient, cluster::ClusterConnection, AsyncCommands,
    Client,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::{Cache, CacheError, CacheManager, CacheResult};

/// Redis配置
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis连接类型
    #[serde(default)]
    pub connection_type: RedisConnectionType,
    /// 连接地址（单机模式）
    #[serde(default = "default_url")]
    pub url: Option<String>,
    /// 主机地址（集群模式）
    #[serde(default)]
    pub hosts: Option<Vec<String>>,
    /// 密码
    #[serde(default)]
    pub password: Option<String>,
    /// 数据库索引（仅单机模式有效）
    #[serde(default = "default_db")]
    pub db: Option<i64>,
    /// 连接池最小空闲连接数
    #[serde(default = "default_min_idle")]
    pub min_idle: Option<u32>,
    /// 连接池最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: Option<u32>,
    /// 连接超时时间（毫秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: Option<u64>,
    /// 命令超时时间（毫秒）
    #[serde(default = "default_command_timeout")]
    pub command_timeout: Option<u64>,
    /// 默认过期时间（秒）
    #[serde(default = "default_default_ttl")]
    pub default_ttl: u64,
}

fn default_url() -> Option<String> {
    Some("redis://127.0.0.1:6379".to_string())
}

fn default_db() -> Option<i64> {
    Some(0)
}

fn default_min_idle() -> Option<u32> {
    Some(5)
}

fn default_max_connections() -> Option<u32> {
    Some(20)
}

fn default_connect_timeout() -> Option<u64> {
    Some(10000)
}

fn default_command_timeout() -> Option<u64> {
    Some(5000)
}

fn default_default_ttl() -> u64 {
    3600
}

/// Redis连接类型
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RedisConnectionType {
    /// 单机模式
    Standalone,
    /// 集群模式
    Cluster,
}

impl Default for RedisConnectionType {
    fn default() -> Self {
        RedisConnectionType::Standalone
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            connection_type: RedisConnectionType::Standalone,
            url: Some("redis://127.0.0.1:6379".to_string()),
            hosts: None,
            password: None,
            db: Some(0),
            min_idle: Some(5),
            max_connections: Some(20),
            connect_timeout: Some(10000),
            command_timeout: Some(5000),
            default_ttl: 3600,
        }
    }
}

/// Redis缓存实现
#[derive(Clone)]
pub struct RedisCache {
    /// 配置
    config: Arc<RedisConfig>,
    /// 单机客户端
    standalone_client: Option<Arc<ConnectionManager>>,
    /// 集群客户端
    cluster_client: Option<Arc<ClusterClient>>,
}

impl RedisCache {
    /// 创建新的Redis缓存实例
    pub async fn new(config: Arc<RedisConfig>) -> CacheResult<Self> {
        match config.connection_type {
            RedisConnectionType::Standalone => {
                let mut url = config.url.clone().ok_or_else(|| {
                    CacheError::Configuration("Redis单机模式需要提供url配置".to_string())
                })?;

                // 处理URL中的认证
                if let Some(password) = &config.password {
                    // 如果URL中没有认证信息，则添加密码
                    if !url.contains("@") {
                        // 解析URL
                        let url_parts: Vec<&str> = url.split("://").collect();
                        if url_parts.len() == 2 {
                            let protocol = url_parts[0];
                            let address = url_parts[1];
                            // 重构URL，添加密码
                            url = format!("{}://:{}@{}", protocol, password, address);
                        }
                    }
                }

                // 如果需要指定数据库，添加到URL
                if let Some(db) = config.db {
                    if !url.contains("/") {
                        url = format!("{}/{}", url, db);
                    }
                }

                // 这里记录没有密码的URL以避免在日志中泄露密码
                let log_url = if url.contains("@") {
                    let url_parts: Vec<&str> = url.split("@").collect();
                    if url_parts.len() > 1 {
                        let protocol_parts: Vec<&str> = url_parts[0].split("://").collect();
                        if protocol_parts.len() > 1 {
                            format!("{}://*****@{}", protocol_parts[0], url_parts[1])
                        } else {
                            "*****@".to_string() + url_parts[1]
                        }
                    } else {
                        "redis://*****".to_string()
                    }
                } else {
                    url.clone()
                };

                info!(
                    "初始化Redis单机连接, URL: {}, 数据库: {:?}",
                    log_url, config.db
                );
                let client = Client::open(url)?;
                let manager = ConnectionManager::new(client).await?;

                Ok(Self {
                    config,
                    standalone_client: Some(Arc::new(manager)),
                    cluster_client: None,
                })
            }
            RedisConnectionType::Cluster => {
                let hosts = config.hosts.clone().ok_or_else(|| {
                    CacheError::Configuration("Redis集群模式需要提供hosts配置".to_string())
                })?;

                // 处理集群连接的认证
                let mut cluster_options = redis::cluster::ClusterClientBuilder::new(hosts.clone());

                // 设置密码认证
                if let Some(password) = &config.password {
                    cluster_options = cluster_options.password(password.clone());
                    info!("Redis集群使用密码认证");
                }

                info!("初始化Redis集群连接, 节点数量: {}", hosts.len());
                let client = cluster_options.build()?;

                Ok(Self {
                    config,
                    standalone_client: None,
                    cluster_client: Some(Arc::new(client)),
                })
            }
        }
    }

    /// 获取单机连接
    async fn get_standalone_conn(&self) -> CacheResult<ConnectionManager> {
        if let Some(client) = &self.standalone_client {
            Ok(client.as_ref().clone())
        } else {
            Err(CacheError::Connection(
                "未初始化Redis单机客户端".to_string(),
            ))
        }
    }

    /// 获取集群连接
    async fn get_cluster_conn(&self) -> CacheResult<ClusterConnection> {
        if let Some(client) = &self.cluster_client {
            client.get_connection().map_err(|e| e.into())
        } else {
            Err(CacheError::Connection(
                "未初始化Redis集群客户端".to_string(),
            ))
        }
    }

    /// 执行Redis命令
    async fn execute<T, F, Fut>(&self, f: F) -> CacheResult<T>
    where
        F: FnOnce(RedisConnection) -> Fut,
        Fut: std::future::Future<Output = redis::RedisResult<T>>,
    {
        let connection = match self.config.connection_type {
            RedisConnectionType::Standalone => {
                RedisConnection::Standalone(self.get_standalone_conn().await?)
            }
            RedisConnectionType::Cluster => {
                // 注意：当前redis-rs的集群实现不支持异步接口
                // 实际使用时可能需要使用单独的线程池处理集群命令
                RedisConnection::Cluster(self.get_cluster_conn().await?)
            }
        };

        f(connection).await.map_err(|e| e.into())
    }
}

/// Redis连接枚举，用于统一单机和集群接口
pub enum RedisConnection {
    Standalone(ConnectionManager),
    Cluster(ClusterConnection),
}

#[async_trait]
impl Cache for RedisCache {
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> CacheResult<()> {
        let serialized = serde_json::to_string(value)?;

        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => conn.set(key, serialized).await,
                RedisConnection::Cluster(ref mut conn) => {
                    redis::cmd("SET").arg(key).arg(serialized).query(conn)
                }
            }
        })
        .await
    }

    async fn set_ex<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<()> {
        let serialized = serde_json::to_string(value)?;
        let seconds = ttl.as_secs() as usize;

        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => {
                    conn.set_ex(key, serialized, seconds).await
                }
                RedisConnection::Cluster(ref mut conn) => redis::cmd("SETEX")
                    .arg(key)
                    .arg(seconds)
                    .arg(serialized)
                    .query(conn),
            }
        })
        .await
    }

    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>> {
        let result: Option<String> = self
            .execute(|mut conn| async move {
                match conn {
                    RedisConnection::Standalone(ref mut conn) => conn.get(key).await,
                    RedisConnection::Cluster(ref mut conn) => {
                        redis::cmd("GET").arg(key).query(conn)
                    }
                }
            })
            .await?;

        match result {
            Some(data) => {
                let value = serde_json::from_str(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let result: Vec<String> = self
            .execute(|mut conn| async move {
                match conn {
                    RedisConnection::Standalone(ref mut conn) => conn.keys(pattern).await,
                    RedisConnection::Cluster(ref mut conn) => {
                        redis::cmd("KEYS").arg(pattern).query(conn)
                    }
                }
            })
            .await?;

        Ok(result)
    }

    async fn del(&self, key: &str) -> CacheResult<()> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => {
                    let _: () = conn.del(key).await?;
                    Ok(())
                }
                RedisConnection::Cluster(ref mut conn) => {
                    let _: () = redis::cmd("DEL").arg(key).query(conn)?;
                    Ok(())
                }
            }
        })
        .await
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => conn.exists(key).await,
                RedisConnection::Cluster(ref mut conn) => redis::cmd("EXISTS").arg(key).query(conn),
            }
        })
        .await
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        let seconds = ttl.as_secs() as usize;

        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => {
                    let _: bool = conn.expire(key, seconds).await?;
                    Ok(())
                }
                RedisConnection::Cluster(ref mut conn) => {
                    let _: bool = redis::cmd("EXPIRE").arg(key).arg(seconds).query(conn)?;
                    Ok(())
                }
            }
        })
        .await
    }

    async fn incr(&self, key: &str) -> CacheResult<i64> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => conn.incr(key, 1).await,
                RedisConnection::Cluster(ref mut conn) => redis::cmd("INCR").arg(key).query(conn),
            }
        })
        .await
    }

    async fn decr(&self, key: &str) -> CacheResult<i64> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => conn.decr(key, 1).await,
                RedisConnection::Cluster(ref mut conn) => redis::cmd("DECR").arg(key).query(conn),
            }
        })
        .await
    }

    // Hash操作方法实现

    async fn hset<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> CacheResult<()> {
        let serialized = serde_json::to_string(value)?;

        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => {
                    let _: () = conn.hset(key, field, serialized).await?;
                    Ok(())
                }
                RedisConnection::Cluster(ref mut conn) => {
                    let _: () = redis::cmd("HSET")
                        .arg(key)
                        .arg(field)
                        .arg(serialized)
                        .query(conn)?;
                    Ok(())
                }
            }
        })
        .await
    }

    async fn hget<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
        field: &str,
    ) -> CacheResult<Option<T>> {
        let result: Option<String> = self
            .execute(|mut conn| async move {
                match conn {
                    RedisConnection::Standalone(ref mut conn) => conn.hget(key, field).await,
                    RedisConnection::Cluster(ref mut conn) => {
                        redis::cmd("HGET").arg(key).arg(field).query(conn)
                    }
                }
            })
            .await?;

        match result {
            Some(data) => {
                let value = serde_json::from_str(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn hdel(&self, key: &str, field: &str) -> CacheResult<()> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => {
                    let _: () = conn.hdel(key, field).await?;
                    Ok(())
                }
                RedisConnection::Cluster(ref mut conn) => {
                    let _: () = redis::cmd("HDEL").arg(key).arg(field).query(conn)?;
                    Ok(())
                }
            }
        })
        .await
    }

    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => conn.hexists(key, field).await,
                RedisConnection::Cluster(ref mut conn) => {
                    redis::cmd("HEXISTS").arg(key).arg(field).query(conn)
                }
            }
        })
        .await
    }

    async fn hkeys(&self, key: &str) -> CacheResult<Vec<String>> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => conn.hkeys(key).await,
                RedisConnection::Cluster(ref mut conn) => redis::cmd("HKEYS").arg(key).query(conn),
            }
        })
        .await
    }

    async fn hlen(&self, key: &str) -> CacheResult<usize> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => conn.hlen(key).await,
                RedisConnection::Cluster(ref mut conn) => redis::cmd("HLEN").arg(key).query(conn),
            }
        })
        .await
    }

    async fn info(&self, key: Option<String>) -> CacheResult<String> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => {
                    if let Some(key) = key {
                        redis::cmd("INFO").arg(key).query_async(conn).await
                    } else {
                        redis::cmd("INFO").query_async(conn).await
                    }
                }
                RedisConnection::Cluster(ref mut conn) => redis::cmd("INFO").query(conn),
            }
        })
        .await
    }

    async fn dbsize(&self) -> CacheResult<usize> {
        self.execute(|mut conn| async move {
            match conn {
                RedisConnection::Standalone(ref mut conn) => {
                    redis::cmd("DBSIZE").query_async(conn).await
                }
                RedisConnection::Cluster(ref mut conn) => redis::cmd("DBSIZE").query(conn),
            }
        })
        .await
    }
}

/// Redis缓存管理器
#[derive(Clone)]
pub struct RedisCacheManager {
    config: Arc<RedisConfig>,
    cache: Arc<RwLock<Option<RedisCache>>>,
}

impl RedisCacheManager {
    /// 创建新的Redis缓存管理器
    pub fn new(config: Arc<RedisConfig>) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn default() -> Self {
        Self {
            config: Arc::new(RedisConfig::default()),
            cache: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl CacheManager for RedisCacheManager {
    type CacheImpl = RedisCache;

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
            let new_cache = RedisCache::new(self.config.clone()).await?;
            *cache_guard = Some(new_cache.clone());
            return Ok(new_cache);
        }

        // 二次检查
        Ok(cache_guard.as_ref().unwrap().clone())
    }
}
