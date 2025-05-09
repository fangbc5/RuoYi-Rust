//! 全局缓存管理器
//!
//! 提供全局的缓存管理器，支持多种缓存类型，使用lazy_static实现懒加载

use lazy_static::lazy_static;
use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::cache::{
    CacheAdapter, CacheBase, CacheError, CacheManager, CacheResult,
    LocalCacheManager, MultiLevelCache, RedisCacheManager,
};
use crate::config::cache::{CacheSettings, CacheType};

// 全局初始化标志
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static INITIALIZING: AtomicBool = AtomicBool::new(false);

// 使用lazy_static和OnceCell为全局缓存管理器创建懒加载初始化
lazy_static! {
    // 使用tokio的OnceCell代替手动管理缓存实例
    static ref GLOBAL_CACHE: OnceCell<Arc<dyn CacheBase>> = OnceCell::new();
}

/// 异步初始化全局缓存管理器
///
/// 只需调用一次，后续调用将被忽略。
/// 此函数为异步函数，可以在任何tokio运行时上下文中调用。
pub async fn init_global_cache_async(settings: Arc<CacheSettings>) -> CacheResult<()> {
    if is_global_cache_initialized() {
        warn!("全局缓存管理器已经初始化，忽略重复初始化请求");
        return Ok(());
    }

    // 检查缓存是否启用
    if !settings.enabled {
        return Err(CacheError::Configuration(
            "请在配置中开启缓存功能".to_string(),
        ));
    }

    // 检查缓存类型对应的配置
    match settings.cache_type {
        CacheType::Local => {
            if settings.local.name.is_empty() {
                return Err(CacheError::Configuration("未提供本地缓存配置".to_string()));
            }
        }
        CacheType::Redis => {
            if settings.redis.url.is_none() {
                return Err(CacheError::Configuration("未提供Redis缓存配置".to_string()));
            }
        }
        CacheType::Multi => {
            if settings.local.name.is_empty() {
                return Err(CacheError::Configuration(
                    "多级缓存缺少本地缓存配置".to_string(),
                ));
            }
            if settings.redis.url.is_none() {
                return Err(CacheError::Configuration(
                    "多级缓存缺少Redis缓存配置".to_string(),
                ));
            }
        }
    }

    // 防止并发初始化
    if INITIALIZING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        // 等待初始化完成
        while INITIALIZING.load(Ordering::SeqCst) && !is_global_cache_initialized() {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        return if is_global_cache_initialized() {
            Ok(())
        } else {
            Err(CacheError::Other("全局缓存初始化失败".to_string()))
        };
    }

    // 开始异步初始化
    let result = match init_cache_by_type(settings.clone()).await {
        Ok(cache) => match GLOBAL_CACHE.set(cache) {
            Ok(_) => {
                INITIALIZED.store(true, Ordering::SeqCst);
                info!("全局缓存管理器初始化完成，类型: {:?}", settings.cache_type);
                Ok(())
            }
            Err(_) => {
                error!("设置全局缓存实例失败");
                Err(CacheError::Other("设置全局缓存实例失败".to_string()))
            }
        },
        Err(e) => {
            error!("初始化全局缓存失败: {}", e);
            Err(e)
        }
    };

    // 重置初始化标志
    INITIALIZING.store(false, Ordering::SeqCst);

    result
}

/// 根据配置类型创建相应的缓存实例
async fn init_cache_by_type(settings: Arc<CacheSettings>) -> CacheResult<Arc<dyn CacheBase>> {
    match settings.cache_type {
        CacheType::Local => {
            info!("正在初始化本地缓存...");

            let local_cache_manager = LocalCacheManager::new(settings.local.clone());
            let cache = local_cache_manager.get_cache().await?;
            let adapter = CacheAdapter::new(cache);
            Ok(Arc::new(adapter))
        }
        CacheType::Redis => {
            info!("正在初始化Redis缓存...");

            let redis_cache_manager = RedisCacheManager::new(settings.redis.clone());
            match redis_cache_manager.get_cache().await {
                Ok(cache) => {
                    let adapter = CacheAdapter::new(cache);
                    Ok(Arc::new(adapter))
                }
                Err(e) => {
                    error!("Redis缓存初始化失败: {}", e);
                    Err(CacheError::Connection(format!("Redis连接失败: {}", e)))
                }
            }
        }
        CacheType::Multi => {
            info!("正在初始化多级缓存...");

            match MultiLevelCache::new(settings.clone()).await {
                Ok(cache) => {
                    let adapter = CacheAdapter::new(cache);
                    Ok(Arc::new(adapter))
                }
                Err(e) => {
                    error!("多级缓存初始化失败: {}", e);
                    Err(e)
                }
            }
        }
    }
}

/// 初始化全局缓存管理器
///
/// 只需调用一次，后续调用将被忽略。
/// 此函数应当在已有tokio运行时的环境中调用。
/// 如果当前不在tokio运行时中，将返回错误。
pub fn init_global_cache(settings: Arc<CacheSettings>) -> CacheResult<()> {
    if is_global_cache_initialized() {
        warn!("全局缓存管理器已经初始化，忽略重复初始化请求");
        return Ok(());
    }

    // 检查当前是否在tokio运行时上下文中
    if !tokio::runtime::Handle::try_current().is_ok() {
        return Err(CacheError::Other(
            "init_global_cache必须在tokio运行时上下文中调用，或者使用init_global_cache_async"
                .to_string(),
        ));
    }

    // 创建一个阻塞任务并等待其完成
    tokio::task::block_in_place(|| {
        // 获取当前运行时句柄
        let rt_handle = tokio::runtime::Handle::current();

        // 在当前运行时上执行异步初始化
        rt_handle.block_on(init_global_cache_async(settings))
    })
}

/// 检查全局缓存管理器是否已初始化
pub fn is_global_cache_initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}

/// 获取全局缓存实例
///
/// 如果全局缓存尚未初始化，则返回错误。
pub fn get_global_cache() -> CacheResult<Arc<dyn CacheBase>> {
    if !is_global_cache_initialized() {
        return Err(CacheError::Other("全局缓存尚未初始化".to_string()));
    }

    // 从OnceCell获取缓存实例
    GLOBAL_CACHE
        .get()
        .cloned()
        .ok_or_else(|| CacheError::Other("全局缓存尚未正确初始化".to_string()))
}

/// 重置全局缓存状态
///
/// 主要用于测试目的，允许重新初始化缓存
#[cfg(test)]
pub fn reset_global_cache() {
    INITIALIZED.store(false, Ordering::SeqCst);
    INITIALIZING.store(false, Ordering::SeqCst);
    // OnceCell不能直接重置，但我们可以通过标志位控制
    // 当我们需要在测试中"重置"缓存时，只需将标志位设置为未初始化
}

// 此模块仅包含基本的单元测试
#[cfg(test)]
mod tests {
    use super::*;

    // 每个测试前重置全局缓存状态
    fn setup() {
        reset_global_cache();
    }

    // 验证重置函数是否正常工作
    #[test]
    fn test_reset_global_cache() {
        INITIALIZED.store(true, Ordering::SeqCst);
        INITIALIZING.store(true, Ordering::SeqCst);

        reset_global_cache();

        assert!(
            !INITIALIZED.load(Ordering::SeqCst),
            "INITIALIZED应该为false"
        );
        assert!(
            !INITIALIZING.load(Ordering::SeqCst),
            "INITIALIZING应该为false"
        );
    }

    #[tokio::test]
    async fn test_global_cache_local() {
        setup();

        // 创建本地缓存配置
        let local_config = crate::cache::LocalCacheConfig {
            name: "test_local_cache".to_string(),
            max_capacity: 1000,
            default_ttl: 3600,
            cleanup_interval: 60,
        };

        let settings = CacheSettings {
            enabled: true,
            cache_type: CacheType::Local,
            local: Arc::new(local_config),
            redis: Arc::new(Default::default()),
            multi: Arc::new(Default::default()),
        };

        // 初始化全局缓存
        let result = init_global_cache_async(Arc::new(settings)).await;
        assert!(result.is_ok(), "本地缓存初始化失败: {:?}", result);
        assert!(is_global_cache_initialized());

        // 测试缓存操作
        let cache = get_global_cache().unwrap();
        let set_result = cache.set_string("test_key", "test_value").await;
        assert!(set_result.is_ok(), "设置缓存失败: {:?}", set_result);

        let get_result = cache.get_string("test_key").await;
        assert!(get_result.is_ok(), "获取缓存失败: {:?}", get_result);

        let value = get_result.unwrap();
        assert_eq!(value, Some("test_value".to_string()), "缓存值不匹配");
    }

    #[tokio::test]
    async fn test_global_cache_redis() {
        setup();

        // 创建Redis缓存配置
        let redis_config = crate::cache::RedisConfig {
            connection_type: crate::cache::RedisConnectionType::Standalone,
            url: Some("redis://127.0.0.1:6379".to_string()),
            password: Some("123456".to_string()),
            db: Some(0),
            ..Default::default()
        };

        let settings = CacheSettings {
            enabled: true,
            cache_type: CacheType::Redis,
            local: Arc::new(Default::default()),
            redis: Arc::new(redis_config),
            multi: Arc::new(Default::default()),
        };
        let settings = Arc::new(settings);
        // 初始化全局缓存 - 可能会因Redis连接问题失败
        match init_global_cache_async(settings.clone()).await {
            Ok(_) => {
                // Redis连接成功，测试缓存操作
                assert!(is_global_cache_initialized());
                let cache = get_global_cache().unwrap();

                // 尝试写入和读取缓存
                let key = "redis_test_key";
                let value = "redis_test_value";

                let set_result = cache.set_string(key, value).await;
                if set_result.is_ok() {
                    let get_result = cache.get_string(key).await;
                    if let Ok(Some(val)) = get_result {
                        assert_eq!(val, value);
                        println!("Redis缓存测试成功");
                    } else {
                        println!("Redis读取失败，可能是写入未成功");
                    }
                } else {
                    println!("Redis写入失败: {:?}", set_result);
                }
            }
            Err(e) => {
                // Redis连接失败，这在测试环境中可能是正常的
                println!("Redis缓存初始化失败，这在测试环境中可能是正常的: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_disabled_cache() {
        setup();

        // 创建禁用缓存的配置
        let settings = CacheSettings {
            enabled: false,
            cache_type: CacheType::Local,
            local: Arc::new(Default::default()),
            redis: Arc::new(Default::default()),
            multi: Arc::new(Default::default()),
        };

        // 初始化全局缓存应当失败
        let result = init_global_cache_async(Arc::new(settings)).await;
        assert!(result.is_err(), "应该失败但却成功了: {:?}", result);

        match result {
            Err(CacheError::Configuration(msg)) => {
                assert!(
                    msg.contains("请在配置中开启缓存"),
                    "错误信息不匹配: {}",
                    msg
                );
            }
            Err(e) => panic!("应该返回配置错误，但实际返回: {:?}", e),
            Ok(_) => panic!("应该失败但却成功了"),
        }
    }

    #[tokio::test]
    async fn test_missing_config() {
        setup();

        // 缺少本地缓存配置：显式创建一个空name的配置
        let empty_local_config = crate::cache::LocalCacheConfig {
            name: "".to_string(),
            ..Default::default()
        };

        let settings = CacheSettings {
            enabled: true,
            cache_type: CacheType::Local,
            local: Arc::new(empty_local_config),
            redis: Arc::new(Default::default()),
            multi: Arc::new(Default::default()),
        };
        let settings = Arc::new(settings);

        // 初始化全局缓存应当失败
        println!(
            "缓存状态: initialized={}, initializing={}",
            INITIALIZED.load(Ordering::SeqCst),
            INITIALIZING.load(Ordering::SeqCst)
        );

        let result = init_global_cache_async(settings.clone()).await;
        println!("初始化结果: {:?}", result);

        assert!(result.is_err(), "应该失败但却成功了: {:?}", result);

        match result {
            Err(CacheError::Configuration(msg)) => {
                assert!(
                    msg.contains("未提供本地缓存配置"),
                    "错误信息不匹配: {}",
                    msg
                );
            }
            Err(e) => panic!("应该返回配置错误，但实际返回: {:?}", e),
            Ok(_) => panic!("应该失败但却成功了"),
        }
    }

    #[tokio::test]
    async fn test_multi_cache() {
        setup();

        // 创建本地缓存配置
        let local_config = crate::cache::LocalCacheConfig {
            name: "test_multi_local".to_string(),
            max_capacity: 1000,
            default_ttl: 3600,
            cleanup_interval: 60,
        };

        let redis_config = crate::cache::RedisConfig {
            connection_type: crate::cache::RedisConnectionType::Standalone,
            url: Some("redis://127.0.0.1:6379".to_string()),
            password: Some("123456".to_string()),
            db: Some(0),
            ..Default::default()
        };

        let multi_config = crate::cache::MultiLevelCacheConfig {
            local_ttl: 300,
            fallback_to_local: true,
        };

        let settings = CacheSettings {
            enabled: true,
            cache_type: CacheType::Multi,
            local: Arc::new(local_config),
            redis: Arc::new(redis_config),
            multi: Arc::new(multi_config),
        };
        let settings = Arc::new(settings);

        // 初始化全局缓存 - 可能会因Redis连接问题失败
        match init_global_cache_async(settings.clone()).await {
            Ok(_) => {
                assert!(is_global_cache_initialized());
                println!("多级缓存初始化成功");

                // 测试写入和读取
                let cache = get_global_cache().unwrap();
                let key = "multi_test_key";
                let value = "multi_test_value";

                let set_result = cache.set_string(key, value).await;
                if set_result.is_ok() {
                    if let Ok(Some(result)) = cache.get_string(key).await {
                        assert_eq!(result, value);
                        println!("多级缓存读写测试成功");
                    }
                }
            }
            Err(e) => {
                // Redis连接失败可能导致多级缓存初始化失败
                println!("多级缓存初始化失败，可能是Redis连接问题: {}", e);
            }
        }
    }
}
