// ruoyi-common/src/utils/http.rs
//! HTTP 工具模块

use actix_web::HttpRequest;
use std::collections::HashMap;

use crate::vo::{RData, RList, R};

/// 获取请求头
pub fn get_header(req: &HttpRequest, header_name: &str) -> Option<String> {
    req.headers()
        .get(header_name)
        .and_then(|h| h.to_str().ok().map(|s| s.to_string()))
}

/// 解析查询参数
pub fn parse_query_string(query_str: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for param in query_str.split('&') {
        if let Some(index) = param.find('=') {
            let key = &param[..index];
            let value = if index < param.len() - 1 {
                &param[index + 1..]
            } else {
                ""
            };

            params.insert(key.to_string(), value.to_string());
        }
    }

    params
}

/// 从请求获取客户端 IP
pub fn get_client_ip(req: &HttpRequest) -> String {
    // 尝试从 X-Forwarded-For 头获取
    if let Some(forwarded) = get_header(req, "X-Forwarded-For") {
        let ips: Vec<&str> = forwarded.split(',').collect();
        if !ips.is_empty() {
            return ips[0].trim().to_string();
        }
    }

    // 尝试从 X-Real-IP 头获取
    if let Some(real_ip) = get_header(req, "X-Real-IP") {
        return real_ip;
    }

    // 获取连接 IP
    match req.connection_info().peer_addr() {
        Some(addr) => addr.to_string(),
        None => "未知".to_string(),
    }
}

/// 构建响应 JSON
pub fn json_ok<T: serde::Serialize>(data: T) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(R::<T>::ok_with_data(data))
}

/// 构建成功响应 JSON
pub fn json_ok_with_data<T: serde::Serialize>(data: T) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(RData::<T>::ok(data))
}

pub fn json_ok_with_list<T: serde::Serialize>(data: Vec<T>) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(RList::<T>::ok_with_data(data))
}

/// 构建错误响应 JSON
pub fn json_error(code: i32, msg: &str) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(R::<String>::error_with_code_msg(code, msg))
}

/// 获取浏览器信息
pub fn get_browser_info(req: &HttpRequest) -> String {
    let ua = get_header(req, "User-Agent").unwrap_or_default();
    // 浏览器快速提取
    let browser = if let Some(_pos) = ua.find("Chrome/") {
        "Chrome"
    } else if let Some(_pos) = ua.find("Safari/") {
        "Safari"
    } else if ua.contains("Firefox") {
        "Firefox"
    } else {
        "Unknown"
    };
    browser.to_string()
}

/// 获取操作系统信息
pub fn get_os_info(req: &HttpRequest) -> String {
    let ua = get_header(req, "User-Agent").unwrap_or_default();
    // 操作系统快速提取
    let os = if ua.contains("Windows NT 10.0") {
        "Windows 10"
    } else if ua.contains("Windows NT 6.3") {
        "Windows 8.1"
    } else if ua.contains("iPhone") {
        "iOS"
    } else if ua.contains("Android") {
        "Android"
    } else if ua.contains("Mac OS X") {
        "macOS"
    } else {
        "Unknown"
    };
    os.to_string()
}

/// 获取请求参数
pub fn get_request_params(req: &HttpRequest) -> Option<String> {
    let mut params = HashMap::new();

    // 获取查询字符串参数
    if let Some(query) = req.uri().query() {
        let query_params = parse_query_string(query);
        for (k, v) in query_params {
            params.insert(k, v);
        }
    }

    // 尝试获取 Content-Type
    let _content_type = get_header(req, "Content-Type").unwrap_or_default();

    // 请求数据体需要在控制器中处理，因为宏无法访问已经消费的请求体
    // 这里我们只返回查询参数

    // 如果有参数，序列化为JSON字符串
    if !params.is_empty() {
        match serde_json::to_string(&params) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    } else {
        None
    }
}
