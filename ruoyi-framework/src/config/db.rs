// ruoyi-framework/src/config/db.rs
//! 数据库配置模块

use serde::Deserialize;

/// 数据库配置详情
#[derive(Debug, Deserialize, Clone)]
pub struct DbSettings {
    /// 数据库连接 URL
    pub url: String,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// 最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// 连接超时时间（秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
    /// 最大生命周期（秒）
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
    /// 空闲超时时间（秒）
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
}

impl Default for DbSettings {
    fn default() -> Self {
        Self {
            url: "mysql://root:password@localhost:3306/ruoyi".to_string(),
            max_connections: 100,
            min_connections: 5,
            connect_timeout: 10,
            max_lifetime: 1800,
            idle_timeout: 600,
        }
    }
}

fn default_max_connections() -> u32 {
    100
}

fn default_min_connections() -> u32 {
    5
}

fn default_connect_timeout() -> u64 {
    10
}

fn default_max_lifetime() -> u64 {
    1800
}

fn default_idle_timeout() -> u64 {
    600
}

impl DbSettings {
    pub fn from_url(url: &str) -> Self {
        Self {
            url: url.to_string(),
            ..Default::default()
        }
    }
}
