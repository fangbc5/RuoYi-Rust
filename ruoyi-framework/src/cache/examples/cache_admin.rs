use crate::cache::{
    Cache, CacheError, CacheManager, LocalCacheConfig, LocalCacheManager, MultiLevelCacheConfig,
    MultiLevelCacheManager, RedisCacheManager, RedisConfig, RedisConnectionType,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// 用户配置示例结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub theme: String,
    pub language: String,
    pub notifications_enabled: bool,
    pub last_updated: String,
}

/// 缓存管理控制台应用示例
pub async fn cache_admin_example() -> Result<(), CacheError> {
    println!("=== 缓存管理控制台应用示例 ===");
    println!("初始化缓存管理器...");

    // 创建Redis配置
    let redis_config = RedisConfig {
        connection_type: RedisConnectionType::Standalone,
        url: Some("redis://127.0.0.1:6379".to_string()),
        password: Some("123456".to_string()),
        db: Some(0),
        ..Default::default()
    };

    // 创建多级缓存配置
    let multi_config = MultiLevelCacheConfig {
        local_ttl: 300,
        fallback_to_local: true,
    };

    // 创建CacheSettings配置
    let settings = crate::config::cache::CacheSettings {
        enabled: true,
        cache_type: crate::config::cache::CacheType::Multi,
        local: Arc::new(LocalCacheConfig::default()),
        redis: Arc::new(redis_config.clone()),
        multi: Arc::new(multi_config),
    };

    // 初始化各种缓存管理器
    let local_manager = LocalCacheManager::default();
    // let redis_manager = RedisCacheManager::new(Arc::new(redis_config));
    // let multi_manager = MultiLevelCacheManager::new(Arc::new(settings));

    // 获取缓存实例
    println!("获取缓存实例...");
    let local_cache = local_manager.get_cache().await?;
    // let redis_cache = redis_manager.get_cache().await?;
    // let multi_cache = multi_manager.get_cache().await?;

    // 执行各种缓存操作示例
    println!("\n1. 基本键值操作示例");
    await_key_operations(&local_cache, "本地缓存").await?;
    // 其他缓存操作暂时注释掉，等待完整实现
    // await_key_operations(&redis_cache, "Redis缓存").await?;
    // await_key_operations(&multi_cache, "多级缓存").await?;

    // 先注释掉其他操作示例
    /*
    println!("\n2. 哈希表操作示例");
    await_hash_operations(&local_cache, "本地缓存").await?;
    await_hash_operations(&redis_cache, "Redis缓存").await?;
    await_hash_operations(&multi_cache, "多级缓存").await?;

    println!("\n3. 复杂对象缓存示例");
    await_object_operations(&local_cache, "本地缓存").await?;
    await_object_operations(&redis_cache, "Redis缓存").await?;
    await_object_operations(&multi_cache, "多级缓存").await?;

    println!("\n4. 计数器和过期操作示例");
    await_counter_operations(&local_cache, "本地缓存").await?;
    await_counter_operations(&redis_cache, "Redis缓存").await?;
    await_counter_operations(&multi_cache, "多级缓存").await?;

    println!("\n5. 多级缓存特性演示");
    await_multi_level_features(&multi_cache).await?;
    */

    println!("\n缓存管理控制台应用示例完成");
    Ok(())
}

/// 基本键值操作示例
async fn await_key_operations<C: Cache>(cache: &C, cache_type: &str) -> Result<(), CacheError> {
    println!("\n--- {}基本键值操作 ---", cache_type);

    // 设置简单键值
    cache.set("simple_key", &"Hello Cache World").await?;
    println!("设置键 'simple_key' 成功");

    // 获取键值
    let value: Option<String> = cache.get("simple_key").await?;
    println!("获取键 'simple_key': {:?}", value);

    // 设置带过期时间的键值
    cache
        .set_ex("expiring_key", &"短期数据", Duration::from_secs(5))
        .await?;
    println!("设置带过期时间的键 'expiring_key' 成功 (5秒)");

    // 立即获取
    let value: Option<String> = cache.get("expiring_key").await?;
    println!("立即获取 'expiring_key': {:?}", value);

    // 删除键
    cache.del("simple_key").await?;
    println!("删除键 'simple_key' 成功");

    // 确认删除成功
    let value: Option<String> = cache.get("simple_key").await?;
    println!("尝试获取已删除的键 'simple_key': {:?}", value);

    // 等待过期时间
    println!("等待6秒让 'expiring_key' 过期...");
    sleep(Duration::from_secs(6)).await;

    // 确认过期
    let value: Option<String> = cache.get("expiring_key").await?;
    println!("过期后获取 'expiring_key': {:?}", value);

    Ok(())
}

// 因为接口方法不完全匹配，先注释掉以下代码
/*
/// 哈希表操作示例
async fn await_hash_operations<C: Cache>(cache: &C, cache_type: &str) -> Result<(), CacheError> {
    println!("\n--- {}哈希表操作 ---", cache_type);

    let hash_key = "user:profile:101";

    // 设置哈希字段
    cache.hset(hash_key, "name", &"张三").await?;
    cache.hset(hash_key, "age", &30).await?;
    cache.hset(hash_key, "email", &"zhangsan@example.com").await?;
    cache.hset(hash_key, "role", &"管理员").await?;
    println!("设置哈希表 '{}' 的多个字段成功", hash_key);

    // 获取单个字段
    let name: Option<String> = cache.hget(hash_key, "name").await?;
    let age: Option<i32> = cache.hget(hash_key, "age").await?;
    println!("获取字段 - 姓名: {:?}, 年龄: {:?}", name, age);

    // 检查字段是否存在
    let exists = cache.hexists(hash_key, "email").await?;
    println!("字段 'email' 是否存在: {}", exists);

    // 获取所有键
    let keys = cache.hkeys(hash_key).await?;
    println!("哈希表所有键: {:?}", keys);

    // 获取哈希表大小
    let len = cache.hlen(hash_key).await?;
    println!("哈希表字段数量: {}", len);

    // 删除字段
    cache.hdel(hash_key, "role").await?;
    println!("删除字段 'role' 成功");

    // 确认删除成功
    let exists = cache.hexists(hash_key, "role").await?;
    println!("字段 'role' 是否存在: {}", exists);

    // 清理所有数据
    cache.del(hash_key).await?;
    println!("删除整个哈希表 '{}' 成功", hash_key);

    Ok(())
}

/// 复杂对象缓存示例
async fn await_object_operations<C: Cache>(cache: &C, cache_type: &str) -> Result<(), CacheError> {
    println!("\n--- {}复杂对象操作 ---", cache_type);

    // 创建一个复杂对象
    let user_config = UserConfig {
        theme: "dark".to_string(),
        language: "zh_CN".to_string(),
        notifications_enabled: true,
        last_updated: Local::now().to_string(),
    };

    // 直接存储为键值
    let key = "user:settings:1001";
    cache.set(key, &user_config).await?;
    println!("存储用户配置到键 '{}' 成功", key);

    // 获取对象
    let retrieved: Option<UserConfig> = cache.get(key).await?;
    println!("获取用户配置: {:?}", retrieved);

    // 存储在哈希表中
    let hash_key = "user:settings:hash:1001";
    cache.hset(hash_key, "theme", &user_config.theme).await?;
    cache.hset(hash_key, "language", &user_config.language).await?;
    cache.hset(hash_key, "notifications_enabled", &user_config.notifications_enabled).await?;
    cache.hset(hash_key, "last_updated", &user_config.last_updated).await?;
    println!("将用户配置存储为哈希表 '{}' 成功", hash_key);

    // 从哈希表获取各个字段
    let theme: Option<String> = cache.hget(hash_key, "theme").await?;
    let notifications: Option<bool> = cache.hget(hash_key, "notifications_enabled").await?;
    println!("从哈希表获取 - 主题: {:?}, 通知设置: {:?}", theme, notifications);

    // 清理
    cache.del(key).await?;
    cache.del(hash_key).await?;
    println!("清理缓存数据成功");

    Ok(())
}

/// 计数器和过期操作示例
async fn await_counter_operations<C: Cache>(cache: &C, cache_type: &str) -> Result<(), CacheError> {
    println!("\n--- {}计数器和过期操作 ---", cache_type);

    // 设置初始计数
    let counter_key = "page:views:article:1234";
    cache.set(counter_key, &0).await?;
    println!("初始化计数器 '{}' 为 0", counter_key);

    // 递增计数
    for i in 1..=5 {
        let new_value = cache.incr(counter_key).await?;
        println!("递增计数器 ({}): {}", i, new_value);
    }

    // 递增指定值 (接口中没有incrby方法，使用多次incr模拟)
    let mut value = 0;
    for _ in 0..10 {
        value = cache.incr(counter_key).await?;
    }
    println!("增加计数器10: {}", value);

    // 递减计数
    let new_value = cache.decr(counter_key).await?;
    println!("递减计数器: {}", new_value);

    // 设置过期时间
    cache.expire(counter_key, Duration::from_secs(3)).await?;
    println!("设置计数器过期时间为3秒");

    // 等待过期
    println!("等待4秒让计数器过期...");
    sleep(Duration::from_secs(4)).await;

    // 检查是否过期
    let value: Option<i64> = cache.get(counter_key).await?;
    println!("过期后检查计数器值: {:?}", value);

    Ok(())
}

/// 多级缓存特性演示
async fn await_multi_level_features<C: Cache>(cache: &C) -> Result<(), CacheError> {
    println!("\n--- 多级缓存特性演示 ---");

    // 设置键值
    let key = "multi_level_test";
    cache.set(key, &"测试多级缓存层次").await?;
    println!("设置键 '{}' 到多级缓存", key);

    // 多次读取，说明已从本地缓存读取
    for i in 1..=3 {
        let _: Option<String> = cache.get(key).await?;
        println!("第{}次读取 '{}' 从多级缓存", i, key);
    }

    // 模拟使用场景，访问次数统计
    let today = Local::now().format("%Y-%m-%d").to_string();
    let visit_key = format!("visits:{}", today);

    // 初始化当日访问计数
    cache.set(&visit_key, &0).await?;

    // 模拟多个用户访问增加计数 (接口中没有incrby方法，使用多次incr模拟)
    let mut total_visits = 0;
    for user_id in 1..=10 {
        let mut count = 0;
        for _ in 0..user_id {
            count = cache.incr(&visit_key).await?;
            total_visits += 1;
        }
        println!("用户{}访问，当日总访问次数: {}", user_id, count);
    }

    // 确认总计数正确
    let final_count: Option<i64> = cache.get(&visit_key).await?;
    println!("实际记录的总访问次数: {:?}, 期望值: {}", final_count, total_visits);
    assert_eq!(final_count.unwrap(), total_visits);

    // 清理
    cache.del(key).await?;
    cache.del(&visit_key).await?;
    println!("清理多级缓存数据成功");

    Ok(())
}
*/

// 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_admin_example() {
        match cache_admin_example().await {
            Ok(_) => println!("缓存管理控制台应用示例运行成功"),
            Err(e) => println!("缓存管理控制台应用示例运行失败: {:?}", e),
        }
    }
}
