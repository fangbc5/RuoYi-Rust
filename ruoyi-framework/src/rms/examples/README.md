# RMS Redis 实现

本目录包含 RMS (Rust Messaging System) 的 Redis 实现。

## 功能特点

- 基于 Redis Pub/Sub 机制
- 支持单条消息发送和接收
- 支持批量消息处理
- 支持消息确认机制
- 支持消息头部传递

## 依赖项

确保在 Cargo.toml 中添加以下依赖：

```toml
[dependencies]
redis = { version = "0.21.5", features = ["tokio-comp", "connection-manager"] }
tokio = { version = "1.17.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1.52"
uuid = { version = "1.0", features = ["v4"] }
thiserror = "1.0"
```

## 运行示例

在运行示例之前，请确保本地 Redis 服务器正在运行。您可以使用以下命令启动 Redis：

```bash
# 使用Docker运行Redis
docker run --name redis -p 6379:6379 -d redis

# 或使用本地安装的Redis
redis-server
```

然后，您可以在您的Rust项目中创建一个简单的可执行文件来运行示例：

```rust
use ruoyi_framework::rms::examples::redis_example;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 运行基本示例
    println!("运行基本示例:");
    redis_example::run_redis_example().await?;
    
    // 运行批量处理示例
    println!("\n运行批量处理示例:");
    redis_example::run_batch_example().await?;
    
    Ok(())
}
```

## 运行测试

测试也需要一个运行中的Redis服务器。您可以使用以下命令运行测试：

```bash
# 进入项目目录
cd ruoyi-rust/ruoyi-framework

# 运行测试
cargo test --test redis_tests
```

所有测试都应该通过，表明Redis实现正常工作。

## 实现细节

### 消息结构

Redis 实现使用 JSON 序列化来传输消息，消息格式如下：

```json
{
  "id": "消息唯一ID",
  "payload": "消息内容",
  "headers": {
    "key1": "value1",
    "key2": "value2"
  }
}
```

### 消息确认

在当前实现中，消息确认是在应用层处理的，而不是使用Redis的确认机制。这是因为Redis Pub/Sub本身没有内置的消息确认功能。

### 批量处理

批量处理通过累积指定数量的消息，然后一次性调用回调函数来实现。这可以提高处理效率，特别是对于需要批量操作的场景。

## 扩展

您可以根据需要扩展此实现，例如：

1. 添加更多错误处理和重试逻辑
2. 实现更复杂的消息路由机制
3. 添加消息过滤功能
4. 实现消息优先级
5. 添加消息持久化支持 