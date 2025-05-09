# RuoYi-Rust 缓存系统

本缓存系统提供了统一的缓存访问接口，支持多种缓存实现，包括本地缓存、Redis缓存和多级缓存等，适用于不同的应用场景。

## 特性

- **统一接口**：所有缓存实现都遵循相同的`Cache`和`CacheManager`接口，便于切换和配置
- **多实现支持**：
  - **本地缓存**：基于`moka`+`dashmap`实现的高性能线程安全的本地缓存
  - **Redis缓存**：支持单机模式和集群模式的Redis缓存
  - **多级缓存**：结合本地缓存和Redis缓存的优势，支持两级缓存策略
- **哈希结构支持**：支持类似Redis哈希表的操作，便于存储结构化数据
- **并发性能优化**：经过多轮性能测试和优化，确保在高并发场景下的稳定性和性能
- **容错降级机制**：当Redis不可用时，多级缓存可以自动降级到本地缓存模式
- **类型安全**：利用Rust的类型系统，提供类型安全的缓存操作

## 性能测试结果

我们进行了多种场景下的性能测试，包括：

1. **基础读写测试**：测试基本的键值对读写性能
2. **高频哈希表测试**：模拟频繁访问用户配置等小体积数据的场景
3. **多级缓存测试**：对比多级缓存与纯本地缓存和纯Redis缓存的性能

关键结果：

- **高频读操作性能**：本地缓存 > 多级缓存 > Redis缓存
- **并发写操作性能**：Redis缓存 > 多级缓存 > 本地缓存
- **每秒操作数(OPS)**：本地缓存约15万OPS，多级缓存约14万OPS，Redis缓存约7万OPS
- **批量操作性能**：Redis的批量操作性能优于本地缓存的批量操作

### 性能数据（20000读+500写）

| 缓存类型 | 总耗时 | 每秒操作数(OPS) | 相对性能 |
|---------|-------|---------------|---------|
| 本地缓存(DashMap) | ~134ms | ~152,800 | 基准值 |
| 多级缓存 | ~145ms | ~141,200 | 本地缓存的92.4% |
| Redis缓存 | ~299ms | ~68,500 | 本地缓存的44.8% |

## 使用场景建议

1. **单机应用**：使用本地缓存获得最佳性能
2. **分布式应用**：
   - 对一致性要求高的数据：使用Redis缓存
   - 对性能和可用性要求高的数据：使用多级缓存
3. **高频读取场景**：使用多级缓存，大部分读取将命中本地缓存
4. **高并发写入场景**：如果写入需要立即对其他实例可见，使用Redis缓存
5. **哈希表存储场景**：使用多级缓存，能够充分利用本地缓存的高性能和Redis的持久化能力

## 多级缓存策略

多级缓存实现了以下策略：

1. **读操作**：优先从本地缓存读取，如果不存在则从Redis读取
2. **写操作**：同时写入本地缓存和Redis缓存，确保一致性
3. **过期时间**：本地缓存TTL通常比Redis短，确保数据最终一致性
4. **降级模式**：当Redis不可用时，可配置降级到纯本地缓存模式

## 示例代码

```rust
// 创建本地缓存
let local_cache = LocalCacheManager::default().get_cache().await?;
local_cache.set("key", &"value").await?;

// 创建Redis缓存
let redis_config = RedisConfig {
    connection_type: RedisConnectionType::Standalone,
    url: Some("redis://127.0.0.1:6379".to_string()),
    password: Some("123456".to_string()),
    db: Some(0),
    ..Default::default()
};
let redis_cache = RedisCacheManager::new(redis_config).get_cache().await?;
redis_cache.set("key", &"value").await?;

// 创建多级缓存
let multi_config = MultiLevelCacheConfig {
    local_config: LocalCacheConfig::default(),
    redis_config: redis_config,
    local_ttl: 300, // 本地缓存5分钟
    fallback_to_local: true,
};
let multi_cache = MultiLevelCacheManager::new(multi_config).get_cache().await?;
multi_cache.set("key", &"value").await?;

// 哈希表操作示例
multi_cache.hset("user:1", "name", &"张三").await?;
multi_cache.hset("user:1", "age", &30).await?;
let name: Option<String> = multi_cache.hget("user:1", "name").await?;
```

## 最佳实践

1. **合理配置TTL**：根据数据的更新频率和重要性配置合适的过期时间
2. **考虑内存占用**：监控本地缓存的内存使用，防止过度缓存导致的内存溢出
3. **使用哈希结构**：对于相关联的数据（如用户配置），使用哈希结构而不是多个独立键
4. **错误处理**：处理缓存操作可能出现的错误，尤其是在分布式环境中
5. **性能监控**：定期监控缓存命中率和操作延迟，及时调整缓存策略

## 未来优化方向

1. **布隆过滤器**：引入布隆过滤器减少缓存穿透
2. **热点数据监控**：自动识别热点数据并调整缓存策略
3. **缓存预热**：系统启动时预加载常用数据
4. **更多缓存实现**：支持更多缓存后端，如Memcached等
5. **事件通知机制**：支持缓存变更通知，用于多实例之间的缓存一致性

### 性能测试
缓存系统提供了一套全面的性能测试工具，可以比较本地缓存和Redis缓存在不同负载下的性能差异。

#### 运行基本性能测试
```bash
# 运行基本性能比较测试
cargo test -p ruoyi-framework -- cache::examples::test::test_compare_performance --nocapture

# 运行本地缓存(DashMap)并发测试
cargo test -p ruoyi-framework -- cache::examples::test::test_dashmap_concurrent_test --nocapture

# 运行Redis并发测试
cargo test -p ruoyi-framework -- cache::examples::test::test_redis_concurrent_test --nocapture
```

#### 运行完整性能测试套件
```bash
# 运行完整测试套件
cargo test -p ruoyi-framework -- cache::examples::test::test_cache_benchmark_suite --nocapture

# 只运行小规模负载测试 (100 并发任务)
TEST_LOAD=small cargo test -p ruoyi-framework -- cache::examples::test::test_cache_benchmark_suite --nocapture

# 只运行中等规模负载测试 (1000 并发任务)
TEST_LOAD=medium cargo test -p ruoyi-framework -- cache::examples::test::test_cache_benchmark_suite --nocapture

# 只运行大规模负载测试 (5000 并发任务)
TEST_LOAD=large cargo test -p ruoyi-framework -- cache::examples::test::test_cache_benchmark_suite --nocapture
```

#### 性能测试结果示例
在一般情况下，本地缓存(DashMap)的性能优于Redis，通常快1.5-10倍，具体取决于网络环境和并发负载。在极高并发情况下，本地缓存的优势更为明显。

测试结果样例（1000并发任务，值大小30字节）：
- **本地缓存(DashMap)**: ~18ms
- **Redis缓存**: ~35ms
- **性能比**: 本地缓存比Redis快约1.9倍

对于不同的值大小，测试结果也会有所差异：
- 小值(10字节): 本地缓存通常比Redis快2-3倍
- 中值(100字节): 本地缓存通常比Redis快1.5-2倍 
- 大值(1000字节): 性能差距可能会缩小，取决于序列化/反序列化的开销 