// ruoyi-common/src/lib.rs
//! 若依管理系统的通用工具模块

pub mod constants;
pub mod entity;
pub mod error;
pub mod utils;
pub mod vo;
pub mod enums;

/// 通用结果类型
pub type Result<T> = std::result::Result<T, error::Error>;

/// 重新导出常用的类型
pub use anyhow;
pub use thiserror;
