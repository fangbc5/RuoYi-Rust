// ruoyi-framework/src/config/server.rs
//! 服务器配置模块

use serde::Deserialize;

/// 服务器配置详情
#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    /// 主机名
    #[serde(default = "default_host")]
    pub host: String,
    /// 端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// 工作线程数（0 表示使用 CPU 核心数）
    #[serde(default = "default_workers")]
    pub workers: usize,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    /// 请求超时时间（秒）
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
    /// 是否启用 HTTPS
    #[serde(default = "default_enable_https")]
    pub enable_https: bool,
    /// SSL 证书路径
    pub ssl_cert: Option<String>,
    /// SSL 密钥路径
    pub ssl_key: Option<String>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_workers() -> usize {
    0
}

fn default_max_connections() -> usize {
    25000
}

fn default_request_timeout() -> u64 {
    30
}

fn default_enable_https() -> bool {
    false
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 0,
            max_connections: 25000,
            request_timeout: 30,
            enable_https: false,
            ssl_cert: None,
            ssl_key: None,
        }
    }
}
