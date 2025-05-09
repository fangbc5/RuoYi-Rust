use serde::Deserialize;

/// JWT 配置
#[derive(Debug, Deserialize, Clone)]
pub struct JwtSettings {
    /// 密钥
    #[serde(default = "default_secret")]
    pub secret: String,
    /// 过期时间（秒）
    #[serde(default = "default_expires_in")]
    pub expires_in: i64,
}

fn default_secret() -> String {
    "ruoyi_rust_secret_key_for_development".to_string()
}

fn default_expires_in() -> i64 {
    86400
}
