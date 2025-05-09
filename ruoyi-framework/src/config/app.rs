// ruoyi-framework/src/config/app.rs
//! 应用配置模块

use serde::Deserialize;

/// 应用基本配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    /// 应用名称
    #[serde(default = "default_name")]
    pub name: String,
    /// 版本号
    #[serde(default = "default_version")]
    pub version: String,
    /// 版权年份
    #[serde(default = "default_copyright_year")]
    pub copyright_year: String,
    /// 是否演示模式
    #[serde(default = "default_demo_enabled")]
    pub demo_enabled: bool,
    /// 上传路径
    #[serde(default = "default_upload_path")]
    pub upload_path: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "RuoYi-Rust".to_string(),
            version: "0.1.0".to_string(),
            copyright_year: "2025".to_string(),
            demo_enabled: false,
            upload_path: "upload".to_string(),
        }
    }
}

fn default_name() -> String {
    "RuoYi-Rust".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_copyright_year() -> String {
    "2025".to_string()
}

fn default_demo_enabled() -> bool {
    false
}

fn default_upload_path() -> String {
    "upload".to_string()
}
