//! 应用程序配置和初始化

use actix_web::{web, App};
use dashmap::DashMap;
use log::info;
use ruoyi_framework::cache::{init_global_cache_async, is_global_cache_initialized};
use ruoyi_framework::db::DbManager;
use ruoyi_framework::web::service::captcha::InMemoryCaptchaService;
use ruoyi_system::repository::config_repository::ConfigRepositoryImpl;
use ruoyi_system::repository::dept_repository::DeptRepositoryImpl;
use ruoyi_system::repository::dict_data_repository::DictDataRepositoryImpl;
use ruoyi_system::repository::dict_type_repository::DictTypeRepositoryImpl;
use ruoyi_system::repository::login_info_repository::LoginInfoRepositoryImpl;
use ruoyi_system::repository::menu_repository::MenuRepositoryImpl;
use ruoyi_system::repository::notice_repository::NoticeRepositoryImpl;
use ruoyi_system::repository::oper_log_repository::OperLogRepositoryImpl;
use ruoyi_system::repository::post_repository::PostRepositoryImpl;
use ruoyi_system::repository::role_repository::RoleRepositoryImpl;
use ruoyi_system::repository::user_repository::UserRepositoryImpl;
use ruoyi_system::service::{
    config_service, dept_service, dict_data_service, dict_type_service, login_info_service,
    menu_service, notice_service, oper_log_service, post_service, role_service, user_service,
};
use std::sync::Arc;

use ruoyi_framework::config::AppConfig;
use ruoyi_framework::web::middleware::{
    auth::Authentication, cors::default_cors, error::ErrorHandling, logger::RequestLogger,
    performance::PerformanceMonitor, response::ResponseWrapper,
};

// 导入自定义中间件
// 导入路由处理函数
use ruoyi_framework::web::controller::common::{captcha_image, health_check};

/// 配置应用程序
pub fn configure_app(
    config: Arc<AppConfig>,
    jwt_secret: String,
    exclude_paths: Vec<String>,
    db_manager: Arc<DbManager>,
    captcha_cache: Arc<DashMap<String, String>>,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    // 创建应用
    let performance_threshold_ms = 500;
    let config_data = web::Data::new(config);
    // 创建验证码服务实现
    let captcha_service = web::Data::new(InMemoryCaptchaService::new(captcha_cache));
    let user_repository = Arc::new(UserRepositoryImpl::new(db_manager.get_connection()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(db_manager.get_connection()));
    let menu_repository = Arc::new(MenuRepositoryImpl::new(db_manager.get_connection()));
    let dept_repository = Arc::new(DeptRepositoryImpl::new(db_manager.get_connection()));
    let config_repository = Arc::new(ConfigRepositoryImpl::new(db_manager.get_connection()));
    let dict_type_repository = Arc::new(DictTypeRepositoryImpl::new(db_manager.get_connection()));
    let dict_data_repository = Arc::new(DictDataRepositoryImpl::new(db_manager.get_connection()));
    let post_repository = Arc::new(PostRepositoryImpl::new(db_manager.get_connection()));
    let notice_repository = Arc::new(NoticeRepositoryImpl::new(db_manager.get_connection()));

    let oper_log_repository = Arc::new(OperLogRepositoryImpl::new(db_manager.get_connection()));
    let login_info_repository = Arc::new(LoginInfoRepositoryImpl::new(db_manager.get_connection()));

    let user_service = web::Data::new(user_service::UserServiceImpl::new(
        user_repository,
        role_repository.clone(),
        menu_repository.clone(),
        dept_repository.clone(),
    ));
    let role_service = web::Data::new(role_service::RoleServiceImpl::new(role_repository));
    let menu_service = web::Data::new(menu_service::MenuServiceImpl::new(menu_repository));
    let dept_service = web::Data::new(dept_service::DeptServiceImpl::new(dept_repository));
    let config_service = web::Data::new(config_service::ConfigServiceImpl::new(config_repository));
    let dict_type_service = web::Data::new(dict_type_service::DictTypeServiceImpl::new(
        dict_type_repository,
    ));
    let dict_data_service = web::Data::new(dict_data_service::DictDataServiceImpl::new(
        dict_data_repository,
    ));
    let post_service = web::Data::new(post_service::PostServiceImpl::new(post_repository));
    let notice_service = web::Data::new(notice_service::NoticeServiceImpl::new(notice_repository));

    let oper_log_service = web::Data::new(oper_log_service::OperLogServiceImpl::new(
        oper_log_repository,
    ));
    let login_info_service = web::Data::new(login_info_service::LoginInfoServiceImpl::new(
        login_info_repository,
    ));

    let app = App::new()
        // 1. 内置日志中间件 (最外层)
        // .wrap(Logger::default())
        // 2. 性能监控中间件 (自定义)
        .wrap(PerformanceMonitor::new(performance_threshold_ms))
        // 3. 自定义日志中间件
        .wrap(RequestLogger::new())
        // 4. CORS 中间件
        .wrap(default_cors())
        // 5. 响应包装中间件
        .wrap(ResponseWrapper::new())
        // 6. 错误处理中间件
        .wrap(ErrorHandling::new())
        // 7. 认证中间件 (最内层)
        .wrap(Authentication::new(jwt_secret, exclude_paths))
        // 注册服务
        .app_data(config_data)
        .app_data(captcha_service)
        .app_data(user_service)
        .app_data(role_service)
        .app_data(menu_service)
        .app_data(dept_service)
        .app_data(config_service)
        .app_data(dict_type_service)
        .app_data(dict_data_service)
        .app_data(post_service)
        .app_data(notice_service)
        .app_data(oper_log_service)
        .app_data(login_info_service)
        // 基础健康检查路由
        .service(health_check)
        // 验证码路由
        .service(captcha_image)
        // API 路由分组
        .configure(register_routes);

    info!("应用程序配置完成，已注册所有中间件和路由");
    app
}

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(ruoyi_framework::web::register_routes);
    cfg.configure(ruoyi_system::register_routes);
}

/// 获取不需要认证的路径列表
pub fn get_exclude_paths() -> Vec<String> {
    vec![
        "/login".to_string(),              // 登录接口
        "/logout".to_string(), // 登出接口（实际上登出需要认证，但若依的前端可能依赖这个行为）
        "/captchaImage".to_string(), // 验证码接口
        "/health".to_string(), // 健康检查接口
        "/favicon.ico".to_string(), // 网站图标
        "/api/v1/public".to_string(), // 公共API
        "/api/common/captcha".to_string(), // 公共验证码API
        // 静态资源
        "/static".to_string(),
        // 系统模块公共API（需要认证的API应当通过权限控制访问）
        "/api/system/menu/user-menu-tree".to_string(),
    ]
}

pub async fn init_global_cache(config: Arc<AppConfig>) {
    // 1. 初始化全局缓存（通常在应用启动时只执行一次）
    if !is_global_cache_initialized() {
        println!("初始化全局缓存...");


        // 使用异步方法初始化全局缓存
        match init_global_cache_async(config.cache.clone()).await {
            Ok(_) => println!("全局缓存初始化成功"),
            Err(e) => {
                println!(
                    "全局缓存初始化失败: {}，但继续执行（可能降级到本地缓存）",
                    e
                );
            }
        }
    } else {
        println!("全局缓存已经初始化");
    }
}