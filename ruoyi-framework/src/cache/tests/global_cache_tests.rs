//! 全局缓存测试模块

use crate::cache::global_cache::{
    get_global_cache, init_global_cache_async, is_global_cache_initialized,
};
use crate::cache::CacheError;
use crate::cache::{reset_global_cache, LocalCacheConfig};
use crate::config::cache::{CacheSettings, CacheType};
use std::sync::atomic::Ordering;

// 每个测试前重置全局缓存状态
fn setup() {
    reset_global_cache();
}

// 按字母顺序执行测试，可以用名称前缀控制执行顺序
// a_ 前缀表示第一批执行的测试
// b_ 前缀表示第二批执行的测试
// c_ 前缀表示第三批执行的测试

#[tokio::test]
async fn a_test_global_cache_local() {
    setup();

    println!("运行本地缓存测试");

    // 创建本地缓存配置
    let local_config = LocalCacheConfig {
        name: "test_local_cache".to_string(),
        max_capacity: 1000,
        default_ttl: 3600,
        cleanup_interval: 60,
    };

    let settings = CacheSettings {
        enabled: true,
        cache_type: CacheType::Local,
        local: Some(local_config),
        redis: None,
        multi: None,
    };

    // 初始化全局缓存
    let result = init_global_cache_async(&settings).await;
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

    // 确保完全复位
    reset_global_cache();
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn b_test_disabled_cache() {
    setup();

    println!("运行禁用缓存测试");
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // 创建禁用缓存的配置
    let settings = CacheSettings {
        enabled: false,
        cache_type: CacheType::Local,
        local: Some(Default::default()),
        redis: None,
        multi: None,
    };

    // 初始化全局缓存应当失败
    let result = init_global_cache_async(&settings).await;
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

    // 确保完全复位
    reset_global_cache();
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn c_test_missing_config() {
    setup();

    println!("运行缺失配置测试");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 缺少本地缓存配置
    let settings = CacheSettings {
        enabled: true,
        cache_type: CacheType::Local,
        local: None,
        redis: None,
        multi: None,
    };

    // 初始化全局缓存应当失败
    println!(
        "测试缺失配置: 缓存状态 initialized={}",
        is_global_cache_initialized()
    );

    let result = init_global_cache_async(&settings).await;
    println!("测试缺失配置: 初始化结果 {:?}", result);

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
