use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::cache::{
    Cache, CacheManager, CacheResult, LocalCacheManager, RedisCacheManager, RedisConfig,
    RedisConnectionType,
};

/// 哈希操作性能测试
///
/// 对比本地缓存和Redis缓存在哈希表操作上的性能差异
pub async fn hash_performance_test() -> CacheResult<()> {
    println!("=== 哈希表操作性能测试 ===");

    // 测试参数
    let hash_keys_count = 50; // 测试的哈希表数量
    let fields_per_hash = 20; // 每个哈希表的字段数量
    let iterations = 5; // 重复测试次数

    // 初始化本地缓存和Redis缓存
    let local_manager = LocalCacheManager::default();
    let local_cache = local_manager.get_cache().await?;

    let redis_config = RedisConfig {
        connection_type: RedisConnectionType::Standalone,
        url: Some("redis://127.0.0.1:6379".to_string()),
        password: Some("123456".to_string()),
        db: Some(0),
        ..Default::default()
    };
    let redis_manager = RedisCacheManager::new(Arc::new(redis_config));
    let redis_cache = redis_manager.get_cache().await?;

    // 预热 - 清除之前可能存在的测试数据
    for i in 0..hash_keys_count {
        let hash_key = format!("hash_perf_test:{}", i);
        local_cache.del(&hash_key).await?;
        redis_cache.del(&hash_key).await?;
    }

    println!("\n1. 测试哈希表写入性能 (hset)");
    let mut local_write_total = Duration::default();
    let mut redis_write_total = Duration::default();

    for iter in 1..=iterations {
        println!("\n迭代 {} / {}", iter, iterations);

        // 测试本地缓存写入
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            for j in 0..fields_per_hash {
                let field = format!("field:{}", j);
                let value = format!("value:{}:{}", i, j);
                local_cache.hset(&hash_key, &field, &value).await?;
            }
        }
        let local_elapsed = start.elapsed();
        local_write_total += local_elapsed;
        println!("本地缓存写入: {} 毫秒", local_elapsed.as_millis());

        // 测试Redis缓存写入
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            for j in 0..fields_per_hash {
                let field = format!("field:{}", j);
                let value = format!("value:{}:{}", i, j);
                redis_cache.hset(&hash_key, &field, &value).await?;
            }
        }
        let redis_elapsed = start.elapsed();
        redis_write_total += redis_elapsed;
        println!("Redis缓存写入: {} 毫秒", redis_elapsed.as_millis());

        println!(
            "写入性能比较: 本地缓存是Redis的 {:.2}倍",
            redis_elapsed.as_secs_f64() / local_elapsed.as_secs_f64()
        );
    }

    println!("\n2. 测试哈希表读取性能 (hget)");
    let mut local_read_total = Duration::default();
    let mut redis_read_total = Duration::default();

    for iter in 1..=iterations {
        println!("\n迭代 {} / {}", iter, iterations);

        // 测试本地缓存读取
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            for j in 0..fields_per_hash {
                let field = format!("field:{}", j);
                let _: Option<String> = local_cache.hget(&hash_key, &field).await?;
            }
        }
        let local_elapsed = start.elapsed();
        local_read_total += local_elapsed;
        println!("本地缓存读取: {} 毫秒", local_elapsed.as_millis());

        // 测试Redis缓存读取
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            for j in 0..fields_per_hash {
                let field = format!("field:{}", j);
                let _: Option<String> = redis_cache.hget(&hash_key, &field).await?;
            }
        }
        let redis_elapsed = start.elapsed();
        redis_read_total += redis_elapsed;
        println!("Redis缓存读取: {} 毫秒", redis_elapsed.as_millis());

        println!(
            "读取性能比较: 本地缓存是Redis的 {:.2}倍",
            redis_elapsed.as_secs_f64() / local_elapsed.as_secs_f64()
        );
    }

    println!("\n3. 测试哈希表其他操作性能 (hexists, hkeys, hlen)");

    // 测试hexists性能
    let mut local_exists_time = Duration::default();
    let mut redis_exists_time = Duration::default();

    for _iter in 1..=iterations {
        // 本地缓存hexists
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            for j in 0..fields_per_hash {
                let field = format!("field:{}", j);
                local_cache.hexists(&hash_key, &field).await?;
            }
        }
        local_exists_time += start.elapsed();

        // Redis缓存hexists
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            for j in 0..fields_per_hash {
                let field = format!("field:{}", j);
                redis_cache.hexists(&hash_key, &field).await?;
            }
        }
        redis_exists_time += start.elapsed();
    }

    println!(
        "hexists操作 - 本地缓存: {} 毫秒, Redis: {} 毫秒, 比例: {:.2}倍",
        local_exists_time.as_millis(),
        redis_exists_time.as_millis(),
        redis_exists_time.as_secs_f64() / local_exists_time.as_secs_f64()
    );

    // 测试hkeys性能
    let mut local_keys_time = Duration::default();
    let mut redis_keys_time = Duration::default();

    for _iter in 1..=iterations {
        // 本地缓存hkeys
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            local_cache.hkeys(&hash_key).await?;
        }
        local_keys_time += start.elapsed();

        // Redis缓存hkeys
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            redis_cache.hkeys(&hash_key).await?;
        }
        redis_keys_time += start.elapsed();
    }

    println!(
        "hkeys操作 - 本地缓存: {} 毫秒, Redis: {} 毫秒, 比例: {:.2}倍",
        local_keys_time.as_millis(),
        redis_keys_time.as_millis(),
        redis_keys_time.as_secs_f64() / local_keys_time.as_secs_f64()
    );

    // 测试hlen性能
    let mut local_len_time = Duration::default();
    let mut redis_len_time = Duration::default();

    for _iter in 1..=iterations {
        // 本地缓存hlen
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            local_cache.hlen(&hash_key).await?;
        }
        local_len_time += start.elapsed();

        // Redis缓存hlen
        let start = Instant::now();
        for i in 0..hash_keys_count {
            let hash_key = format!("hash_perf_test:{}", i);
            redis_cache.hlen(&hash_key).await?;
        }
        redis_len_time += start.elapsed();
    }

    println!(
        "hlen操作 - 本地缓存: {} 毫秒, Redis: {} 毫秒, 比例: {:.2}倍",
        local_len_time.as_millis(),
        redis_len_time.as_millis(),
        redis_len_time.as_secs_f64() / local_len_time.as_secs_f64()
    );

    // 汇总性能比较结果
    println!("\n === 性能测试汇总 ===");
    println!("操作类型\t本地缓存(ms)\tRedis(ms)\t本地/Redis比例");
    println!(
        "hset\t\t{}\t\t{}\t\t{:.2}倍",
        local_write_total.as_millis(),
        redis_write_total.as_millis(),
        redis_write_total.as_secs_f64() / local_write_total.as_secs_f64()
    );
    println!(
        "hget\t\t{}\t\t{}\t\t{:.2}倍",
        local_read_total.as_millis(),
        redis_read_total.as_millis(),
        redis_read_total.as_secs_f64() / local_read_total.as_secs_f64()
    );
    println!(
        "hexists\t\t{}\t\t{}\t\t{:.2}倍",
        local_exists_time.as_millis(),
        redis_exists_time.as_millis(),
        redis_exists_time.as_secs_f64() / local_exists_time.as_secs_f64()
    );
    println!(
        "hkeys\t\t{}\t\t{}\t\t{:.2}倍",
        local_keys_time.as_millis(),
        redis_keys_time.as_millis(),
        redis_keys_time.as_secs_f64() / local_keys_time.as_secs_f64()
    );
    println!(
        "hlen\t\t{}\t\t{}\t\t{:.2}倍",
        local_len_time.as_millis(),
        redis_len_time.as_millis(),
        redis_len_time.as_secs_f64() / local_len_time.as_secs_f64()
    );

    // 最后清理测试数据
    for i in 0..hash_keys_count {
        let hash_key = format!("hash_perf_test:{}", i);
        local_cache.del(&hash_key).await?;
        redis_cache.del(&hash_key).await?;
    }

    println!("\n测试完成，已清理测试数据");

    Ok(())
}

// 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_performance() {
        match hash_performance_test().await {
            Ok(_) => println!("哈希表性能测试完成"),
            Err(e) => println!("哈希表性能测试失败: {:?}", e),
        }
    }
}
