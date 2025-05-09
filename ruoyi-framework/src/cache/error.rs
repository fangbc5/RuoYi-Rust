//! 缓存错误处理模块

use std::fmt;

/// 缓存操作结果类型
pub type CacheResult<T> = Result<T, CacheError>;

/// 缓存操作错误
#[derive(Debug)]
pub enum CacheError {
    /// Redis错误
    Redis(String),
    /// 序列化错误
    Serialization(String),
    /// 反序列化错误
    Deserialization(String),
    /// 连接错误
    Connection(String),
    /// 配置错误
    Configuration(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::Redis(msg) => write!(f, "Redis错误: {}", msg),
            CacheError::Serialization(msg) => write!(f, "序列化错误: {}", msg),
            CacheError::Deserialization(msg) => write!(f, "反序列化错误: {}", msg),
            CacheError::Connection(msg) => write!(f, "连接错误: {}", msg),
            CacheError::Configuration(msg) => write!(f, "配置错误: {}", msg),
            CacheError::Other(msg) => write!(f, "缓存错误: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        CacheError::Redis(err.to_string())
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::Serialization(err.to_string())
    }
}
