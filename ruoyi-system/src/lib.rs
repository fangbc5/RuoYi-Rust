// ruoyi-system/src/lib.rs
//! 若依管理系统的系统模块

use actix_web::web;

pub mod controller;
pub mod entity;
pub mod repository;
pub mod service;

/// 注册系统模块的所有路由
pub fn register_routes(cfg: &mut web::ServiceConfig) {
    // 注册登录路由
    cfg.configure(controller::login_controller::load_login_routes);
    // 注册路由
    cfg.service(
        web::scope("/system")
            .configure(controller::user_controller::load_user_routes)
            .configure(controller::role_controller::load_role_routes)
            .configure(controller::menu_controller::load_menu_routes)
            .configure(controller::dept_controller::load_dept_routes)
            .configure(controller::post_controller::load_post_routes)
            .configure(controller::config_controller::load_config_routes)
            .configure(controller::dict_data_controller::load_dict_data_routes)
            .configure(controller::dict_type_controller::load_dict_type_routes)
            .configure(controller::notice_controller::load_notice_routes),
    )
    .service(
        web::scope("/monitor")
            .configure(controller::login_info_controller::load_login_info_routes)
            .configure(controller::oper_log_controller::load_oper_log_routes)
            .configure(controller::monitor::cache_controller::load_cache_routes)
            .configure(controller::monitor::server_controller::load_server_routes)
            .configure(controller::monitor::user_online_controller::load_user_online_routes),
    );
}
