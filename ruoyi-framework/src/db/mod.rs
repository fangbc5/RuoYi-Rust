// ruoyi-framework/src/db/mod.rs
//! 数据库模块，负责数据库连接和操作

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::sync::Arc;
use std::time::Duration;

use crate::config::db::DbSettings;

pub mod repository;
pub mod transaction;

/// 数据库连接管理器
#[derive(Clone)]
pub struct DbManager {
    /// 数据库连接
    conn: Arc<DatabaseConnection>,
    /// 数据库配置
    config: Arc<DbSettings>,
}

impl DbManager {
    /// 创建数据库连接管理器
    pub async fn new(config: DbSettings) -> Result<Self, DbErr> {
        let config = Arc::new(config);
        let conn = Self::create_connection(&config).await?;

        Ok(Self { conn: Arc::new(conn), config })
    }

    /// 创建数据库连接
    async fn create_connection(config: &DbSettings) -> Result<DatabaseConnection, DbErr> {
        let mut opt = ConnectOptions::new(config.url.clone());

        opt.max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .sqlx_logging(true);

        Database::connect(opt).await
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> Arc<DatabaseConnection> {
        self.conn.clone()
    }

    /// 获取数据库配置
    pub fn get_config(&self) -> Arc<DbSettings> {
        self.config.clone()
    }
}
