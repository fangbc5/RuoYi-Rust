// ruoyi-common/src/error.rs
//! 错误处理模块

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
#[cfg(feature = "redis")]
use redis;
use serde::Serialize;
use thiserror::Error;

/// 系统错误定义
#[derive(Debug, Error)]
pub enum Error {
    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// 未授权错误
    #[error("未授权: {0}")]
    Unauthorized(String),

    /// 权限不足错误
    #[error("权限不足: {0}")]
    Forbidden(String),

    /// 未找到资源错误
    #[error("未找到资源: {0}")]
    NotFound(String),

    /// 验证错误
    #[error("验证错误: {0}")]
    Validation(String),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 配置错误
    #[error("配置错误: {0}")]
    Configuration(String),

    /// Redis 错误
    #[error("Redis 错误: {0}")]
    #[cfg(feature = "redis")]
    Redis(#[from] redis::RedisError),

    /// JWT 错误
    #[error("JWT 错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// 密码错误
    #[error("密码错误: {0}")]
    PasswordError(String),

    /// 未知错误
    #[error("未知错误: {0}")]
    Unknown(#[from] anyhow::Error),

    /// 业务错误
    #[error("业务错误: {0}")]
    BusinessError(String),

    /// 服务器内部错误
    #[error("服务器内部错误: {0}")]
    InternalServerError(String),

}

/// 错误响应结构
#[derive(Serialize)]
struct ErrorResponse {
    /// 错误码
    code: i32,
    /// 错误消息
    message: String,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16() as i32,
            message: self.to_string(),
        };

        HttpResponse::build(status_code).json(error_response)
    }
}


impl From<argon2::password_hash::Error> for Error {
    fn from(error: argon2::password_hash::Error) -> Self {
        Error::PasswordError(error.to_string())
    }
}
