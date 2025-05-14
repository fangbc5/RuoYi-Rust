mod config;
mod controller;
pub mod entity;
mod repository;
mod service;
mod common;

use actix_web::web;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// 使用sea-orm生成entity命令
/// sea-orm-cli generate entity --tables gen_table,gen_table_column --output-dir src/entity --with-serde both --database-url mysql://root:123456@localhost:3306/ruoyi

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    controller::config(cfg);
}

/// 初始化代码生成模块
pub fn init(db: Arc<DatabaseConnection>) -> web::Data<controller::GenController> {
    // 初始化仓库
    let gen_table_repository = Arc::new(repository::GenTableRepositoryImpl::new(db.clone()));
    let gen_table_column_repository =
        Arc::new(repository::GenTableColumnRepositoryImpl::new(db.clone()));

    // 初始化服务
    let gen_table_service = Arc::new(service::GenTableServiceImpl::new(
        db.clone(),
        gen_table_repository,
        gen_table_column_repository.clone(),
    ));

    // 初始化控制器
    web::Data::new(controller::GenController::new(gen_table_service))
}
