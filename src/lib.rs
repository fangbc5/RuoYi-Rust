// ruoyi-rust/src/lib.rs
//! 若依管理系统的 Rust 实现版本

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用名称
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/*
sea-orm-cli generate entity \
--tables sys_login_info,sys_oper_log \
--output-dir ./src/entity \
--with-serde both \
--database-url mysql://root:123456@localhost:3306/ruoyi
*/