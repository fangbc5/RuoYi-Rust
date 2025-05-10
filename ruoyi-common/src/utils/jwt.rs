// ruoyi-common/src/utils/jwt.rs
//! JWT 工具模块，用于生成和验证 JWT 令牌

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

/// JWT声明信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 主题
    pub sub: String,
    /// 过期时间
    pub exp: i64,
    /// 签发时间
    pub iat: i64,
    /// 用户ID
    pub user_id: i64,
    /// 用户名
    pub user_name: String,
    /// 会话ID
    pub token_id: String,
}

/// 生成 JWT 令牌
pub fn generate_token(
    token_id: &str,
    user_id: i64,
    user_name: &str,
    secret: &str,
    expires_in: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let current_time = Utc::now();
    let exp_time = current_time + Duration::seconds(expires_in);

    let claims = Claims {
        sub: "auth".to_string(),
        exp: exp_time.timestamp(),
        iat: current_time.timestamp(),
        user_id,
        user_name: user_name.to_string(),
        token_id: token_id.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// 验证令牌是否有效
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = jsonwebtoken::Validation::default();
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());

    let token_data = jsonwebtoken::decode::<Claims>(token, &key, &validation)?;

    Ok(token_data.claims)
}
