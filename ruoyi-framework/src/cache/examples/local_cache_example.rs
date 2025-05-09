use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::cache::{
    local_cache::{LocalCacheConfig, LocalCacheManager},
    Cache, CacheManager, CacheResult,
};

/// 用户信息示例结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
}

/// 使用本地缓存示例
pub async fn local_cache_example() -> CacheResult<()> {
    println!("=== 本地缓存示例 ===");

    // 创建本地缓存管理器
    let config = LocalCacheConfig {
        name: "user_cache".to_string(),
        max_capacity: 1000,
        default_ttl: 300, // 5分钟
        ..Default::default()
    };

    let cache_manager = LocalCacheManager::new(Arc::new(config));
    let cache = cache_manager.get_cache().await?;

    // 创建用户示例
    let user = UserInfo {
        id: 1,
        username: "admin".to_string(),
        email: Some("admin@example.com".to_string()),
    };

    // 设置缓存
    let key = format!("user:{}", user.id);
    cache.set(&key, &user).await?;
    println!("已缓存用户: {}", user.username);

    // 读取缓存
    let cached_user: Option<UserInfo> = cache.get(&key).await?;
    if let Some(u) = cached_user {
        println!("从缓存读取用户: {:?}", u);
    }

    // 设置有过期时间的缓存
    let ttl = Duration::from_secs(60); // 1分钟
    cache
        .set_ex(&format!("temp:user:{}", user.id), &user, ttl)
        .await?;

    // 删除缓存
    cache.del(&key).await?;
    println!("已删除缓存");

    // 验证缓存已删除
    let deleted_user: Option<UserInfo> = cache.get(&key).await?;
    println!("删除后查询结果: {:?}", deleted_user);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_cache_example() {
        match local_cache_example().await {
            Ok(_) => println!("本地缓存示例运行成功"),
            Err(e) => println!("本地缓存示例运行失败: {:?}", e),
        }
    }
}
