//! 跨域资源共享中间件

use actix_cors::Cors;
use actix_web::http::header;

/// 创建 CORS 中间件
///
/// # Arguments
///
/// * `allowed_origins` - 允许的源
/// * `max_age` - 缓存有效期
///
/// # Returns
///
/// 返回配置好的 CORS 中间件
pub fn cors_middleware(allowed_origins: Vec<String>, max_age: Option<usize>) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
        .supports_credentials();

    // 配置允许的源
    if allowed_origins.is_empty() {
        // 如果未指定，则允许任何源
        cors = cors.allow_any_origin();
    } else {
        for origin in allowed_origins {
            cors = cors.allowed_origin(&origin);
        }
    }

    // 配置缓存有效期
    if let Some(age) = max_age {
        cors = cors.max_age(age);
    } else {
        cors = cors.max_age(3600); // 默认1小时
    }

    cors
}

/// 创建默认的 CORS 中间件（允许任何源）
pub fn default_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
        .max_age(3600)
        .supports_credentials()
}
