// ruoyi-common/src/utils/ip.rs
//! IP 地址处理工具模块

use actix_web::dev::ServiceRequest;
use actix_web::HttpRequest;
use std::net::IpAddr;
use std::str::FromStr;

const X_FORWARDED_FOR: &str = "X-Forwarded-For";
const X_REAL_IP: &str = "X-Real-IP";
const UNKNOWN: &str = "unknown";


/// 获取请求的ip
/// /// 获取客户端 IP 地址
pub fn get_real_ip_by_middleware(req: &ServiceRequest) -> String {
    // 尝试从 X-Forwarded-For 头获取
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(value) = forwarded.to_str() {
            let ips: Vec<&str> = value.split(',').collect();
            if !ips.is_empty() {
                return ips[0].trim().to_string();
            }
        }
    }

    // 尝试从 X-Real-IP 头获取
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(value) = real_ip.to_str() {
            return value.trim().to_string();
        }
    }

    // 获取连接的远程地址
    match req.connection_info().peer_addr() {
        Some(addr) => addr.to_string(),
        None => "未知".to_string(),
    }
}
/// 获取请求的真实IP地址
pub fn get_real_ip_by_request(req: &HttpRequest) -> String {
    // 先尝试从 X-Forwarded-For 获取
    if let Some(x_forwarded_for) = req.headers().get(X_FORWARDED_FOR) {
        if let Ok(value) = x_forwarded_for.to_str() {
            let ips: Vec<&str> = value.split(',').collect();
            if !ips.is_empty() && ips[0] != UNKNOWN {
                return ips[0].trim().to_string();
            }
        }
    }

    // 再尝试从 X-Real-IP 获取
    if let Some(x_real_ip) = req.headers().get(X_REAL_IP) {
        if let Ok(value) = x_real_ip.to_str() {
            if value != UNKNOWN {
                return value.trim().to_string();
            }
        }
    }

    // 最后获取连接的远程地址
    match req.connection_info().peer_addr() {
        Some(addr) => {
            // 解析出IP部分，去掉可能的端口
            if let Some(colon_idx) = addr.rfind(':') {
                addr[..colon_idx].to_string()
            } else {
                addr.to_string()
            }
        }
        None => UNKNOWN.to_string(),
    }
}

/// 判断是否为内网IP
pub fn is_internal_ip(ip: &str) -> bool {
    if let Ok(ip_addr) = IpAddr::from_str(ip) {
        match ip_addr {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();

                // 10.0.0.0/8
                if octets[0] == 10 {
                    return true;
                }

                // 172.16.0.0/12
                if octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31) {
                    return true;
                }

                // 192.168.0.0/16
                if octets[0] == 192 && octets[1] == 168 {
                    return true;
                }

                // 127.0.0.0/8
                if octets[0] == 127 {
                    return true;
                }
            }
            IpAddr::V6(ipv6) => {
                // IPv6 本地地址
                return ipv6.is_loopback() || ipv6.is_unspecified();
            }
        }
    }

    false
}

/// 获取 IP 所在地理位置
pub fn get_ip_location(ip: &str) -> String {
    // 本地开发阶段直接返回内网地址
    if ip.is_empty() || UNKNOWN == ip || is_internal_ip(ip) {
        return "内网IP".to_string();
    }

    // TODO: 实际项目中可以接入 IP 地址库或第三方服务查询 IP 地址所在地理位置
    format!("IP: {}", ip)
}
