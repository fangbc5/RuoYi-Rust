// ruoyi-framework/src/config/mod.rs
//! 配置模块，负责加载和管理系统配置

use app::AppSettings;
use cache::CacheSettings;
use config::{Config, ConfigError, Environment, File};
use db::DbSettings;
use dotenv::dotenv;
use jwt::JwtSettings;
use serde::Deserialize;
use server::ServerSettings;
use std::env;
use std::path::Path;
use std::sync::Arc;

pub mod app;
pub mod cache;
pub mod db;
pub mod jwt;
pub mod server;

/// 应用配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// 应用名称
    pub app: Arc<AppSettings>,
    /// JWT 配置
    pub jwt: Arc<JwtSettings>,
    /// 服务器配置
    pub server: Arc<ServerSettings>,
    /// 数据库配置
    pub database: Arc<DbSettings>,
    /// Redis 配置
    pub cache: Arc<CacheSettings>,
}

/// 加载配置
pub fn load_config() -> Result<Arc<AppConfig>, ConfigError> {
    // 加载 .env 文件
    dotenv().ok();

    // 确定配置文件路径
    let config_dir = env::var("CONFIG_PATH").unwrap_or_else(|_| "config".to_string());
    let env = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

    let config_path = Path::new(&config_dir);
    let default_path = config_path.join("default.toml");
    let env_path = config_path.join(format!("{}.toml", env));

    // 创建配置
    let config = Config::builder()
        // 加载默认配置
        .add_source(File::from(default_path).required(false))
        // 加载环境特定配置
        .add_source(File::from(env_path).required(false))
        // 加载环境变量
        .add_source(Environment::with_prefix("RUOYI").separator("__"))
        .build()?;

    // 反序列化为 AppConfig
    let app_config: AppConfig = config.try_deserialize()?;
    Ok(Arc::new(app_config))
}
