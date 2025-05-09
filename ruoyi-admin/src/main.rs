//! 若依管理系统的 Rust 实现版本的主程序
mod app;

use actix_web::HttpServer;
use app::init_global_cache;
use dashmap::DashMap;
use log::info;
use ruoyi_framework::config::{db::DbSettings, load_config};
use ruoyi_framework::db::DbManager;
use ruoyi_framework::logger::init_logger_with_db;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载配置
    let app_config = load_config().expect("加载配置失败");

    println!("若依管理系统启动中...");

    let jwt_secret = app_config.jwt.secret.clone();

    // 获取服务器配置
    let host = &app_config.server.host;
    let port = &app_config.server.port;
    let server_url = format!("{}:{}", host, port);

    // 获取排除认证的路径
    let exclude_paths = app::get_exclude_paths();

    // 获取数据库连接
    let db_settings = DbSettings::from_url(&app_config.database.url);
    let db_manager = match DbManager::new(db_settings).await {
        Ok(manager) => {
            println!("数据库连接初始化成功");
            Arc::new(manager)
        }
        Err(e) => {
            println!("数据库连接初始化失败: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };
    
    // 初始化日志系统
    init_logger_with_db(db_manager.get_connection()).expect("初始化日志系统失败");
    init_global_cache(app_config.clone()).await;

    let captcha_cache = Arc::new(DashMap::new());

    info!("服务器地址: http://{}", server_url);
    info!("排除认证的路径: {:?}", exclude_paths);

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        // 配置应用
        app::configure_app(
            app_config.clone(),
            jwt_secret.clone(),
            exclude_paths.clone(),
            db_manager.clone(),
            captcha_cache.clone(),
        )
    })
    .bind(server_url)?
    .workers(4) // 设置工作线程数
    .run()
    .await
}
