// ruoyi-framework/src/middleware/auth.rs
//! 身份验证中间件

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use log::{debug, error};
use std::sync::Arc;

use ruoyi_common::utils::jwt::validate_token;

use crate::web::tls;

/// 认证中间件
pub struct Authentication {
    /// JWT 密钥
    pub jwt_secret: String,
    /// 不需要认证的路径
    pub exclude_paths: Vec<String>,
}

impl Authentication {
    /// 创建认证中间件
    pub fn new(jwt_secret: String, exclude_paths: Vec<String>) -> Self {
        Self {
            jwt_secret,
            exclude_paths,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service,
            jwt_secret: self.jwt_secret.clone(),
            exclude_paths: self.exclude_paths.clone(),
        }))
    }
}

/// 认证中间件实现
pub struct AuthenticationMiddleware<S> {
    /// 服务
    service: S,
    /// JWT 密钥
    jwt_secret: String,
    /// 不需要认证的路径
    exclude_paths: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        debug!("处理认证请求: {}", path);

        // 检查是否需要认证
        if self.exclude_paths.iter().any(|p| path.starts_with(p)) {
            debug!("路径 {} 不需要认证", path);
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }
        // 获取 Authorization 头
        let auth_header = req.headers().get("Authorization");

        // 验证令牌格式
        let token = match auth_header {
            Some(header) => {
                let header_str = match header.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        error!("无效的认证头: {:?}", header);
                        return Box::pin(async move {
                            Err(actix_web::error::ErrorUnauthorized("无效的认证头"))
                        });
                    }
                };

                if header_str.starts_with("Bearer ") {
                    header_str[7..].to_string()
                } else {
                    error!("认证头格式错误: {}", header_str);
                    return Box::pin(async move {
                        Err(actix_web::error::ErrorUnauthorized(
                            "未提供有效的Bearer令牌",
                        ))
                    });
                }
            }
            None => {
                error!("请求未提供认证令牌: {}", path);
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("未提供认证令牌"))
                });
            }
        };

        // 验证 JWT 令牌
        let claims = match validate_token(&token, &self.jwt_secret) {
            Ok(data) => data,
            Err(e) => {
                error!("无效的认证令牌: {}", e);
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("无效的认证令牌"))
                });
            }
        };

        // 将用户信息添加到请求扩展中
        let claims = Arc::new(claims);
        req.extensions_mut().insert(claims.clone());

        // 获取ip地址
        let ip = req.peer_addr().unwrap().ip();
        // 设置用户上下文
        tls::set_sync_user_context(tls::UserContext {
            user_id: claims.user_id,
            user_name: claims.user_name.clone(),
            ip,
        });
        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}
