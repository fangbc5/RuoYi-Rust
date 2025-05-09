//! 全局缓存使用示例

use crate::cache::{
    get_global_cache, init_global_cache_async, is_global_cache_initialized, CacheResult,
    LocalCacheConfig, MultiLevelCacheConfig, RedisConfig, RedisConnectionType,
};
use crate::config::cache::{CacheSettings, CacheType};
use std::sync::Arc;

/// 全局缓存使用示例
///
/// 这个示例展示了如何初始化全局缓存并在应用程序的不同部分使用它
pub async fn global_cache_example() -> CacheResult<()> {
    println!("=== 全局缓存使用示例 ===");

    // 1. 初始化全局缓存（通常在应用启动时只执行一次）
    if !is_global_cache_initialized() {
        println!("初始化全局缓存...");

        // 创建本地缓存配置
        let local_config = LocalCacheConfig {
            name: "global".to_string(),
            max_capacity: 10000,
            default_ttl: 3600,
            cleanup_interval: 60,
        };

        // 创建Redis配置
        let redis_config = RedisConfig {
            connection_type: RedisConnectionType::Standalone,
            url: Some("redis://127.0.0.1:6379".to_string()),
            password: None,
            db: Some(0),
            ..Default::default()
        };

        // 创建多级缓存配置
        let multi_config = MultiLevelCacheConfig {
            local_ttl: 300,          // 本地缓存5分钟
            fallback_to_local: true, // 允许降级到本地缓存
        };

        // 创建CacheSettings配置
        let settings = CacheSettings {
            enabled: true,
            cache_type: CacheType::Multi,
            local: Arc::new(local_config),
            redis: Arc::new(redis_config),
            multi: Arc::new(multi_config),
        };

        // 使用异步方法初始化全局缓存
        match init_global_cache_async(Arc::new(settings)).await {
            Ok(_) => println!("全局缓存初始化成功"),
            Err(e) => {
                println!(
                    "全局缓存初始化失败: {}，但继续执行（可能降级到本地缓存）",
                    e
                );
            }
        }
    } else {
        println!("全局缓存已经初始化");
    }

    // 2. 在应用程序的不同部分使用全局缓存
    // 模拟用户服务
    user_service_example().await?;

    // 模拟系统配置服务
    config_service_example().await?;

    println!("全局缓存示例完成");
    Ok(())
}

/// 模拟用户服务使用全局缓存
async fn user_service_example() -> CacheResult<()> {
    println!("\n模拟用户服务使用全局缓存:");

    // 获取全局缓存
    let cache = get_global_cache()?;

    // 用户数据
    let user_id = "1001";
    let user_key = format!("user:{}", user_id);
    let user_data = "张三,30,男,工程师";

    // 缓存用户数据
    cache.set_string(&user_key, user_data).await?;
    println!("缓存用户数据: {}", user_data);

    // 读取用户数据
    let cached_user = cache.get_string(&user_key).await?;
    println!("读取缓存的用户数据: {:?}", cached_user);

    // 哈希表存储用户偏好设置
    let pref_key = format!("user:pref:{}", user_id);

    // 设置用户偏好
    cache.hset_string(&pref_key, "theme", "dark").await?;
    cache.hset_string(&pref_key, "language", "zh_CN").await?;
    // 使用布尔值
    cache.hset_int(&pref_key, "notifications", 1).await?;

    // 读取用户偏好
    let theme = cache.hget_string(&pref_key, "theme").await?;
    let lang = cache.hget_string(&pref_key, "language").await?;
    let notifications = cache.hget_int(&pref_key, "notifications").await?;

    println!(
        "用户偏好设置: theme={:?}, language={:?}, notifications={:?}",
        theme, lang, notifications
    );

    Ok(())
}

/// 模拟系统配置服务使用全局缓存
async fn config_service_example() -> CacheResult<()> {
    println!("\n模拟系统配置服务使用全局缓存:");

    // 获取全局缓存
    let cache = get_global_cache()?;

    // 系统配置键
    let config_key = "system:config";

    // 设置系统配置
    cache
        .hset_string(config_key, "max_upload_size", "50MB")
        .await?;
    // 使用整数代替布尔值
    cache.hset_int(config_key, "enable_registration", 1).await?;
    cache.hset_int(config_key, "session_timeout", 1800).await?;

    // 读取系统配置
    let max_size = cache.hget_string(config_key, "max_upload_size").await?;
    let enable_reg = cache.hget_int(config_key, "enable_registration").await?;
    let timeout = cache.hget_int(config_key, "session_timeout").await?;

    println!(
        "系统配置: max_upload_size={:?}, enable_registration={:?}, session_timeout={:?}秒",
        max_size, enable_reg, timeout
    );

    // 获取所有配置项
    let all_keys = cache.hkeys(config_key).await?;
    println!("所有配置项: {:?}", all_keys);

    // 配置项数量
    let count = cache.hlen(config_key).await?;
    println!("配置项数量: {}", count);

    Ok(())
}

// 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_global_cache_example() {
        match global_cache_example().await {
            Ok(_) => println!("全局缓存示例测试成功"),
            Err(e) => println!("全局缓存示例测试失败: {:?}", e),
        }
    }
}
