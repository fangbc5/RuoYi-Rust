# RMS - Rust Message Service

RMS 是一个类似于 JMS (Java Message Service) 和 Spring Stream 的消息系统抽象层，为不同的消息队列实现提供统一的接口。

## 设计目标

1. **抽象统一**：提供一致的接口，使应用可以轻松切换不同的消息队列实现
2. **简单易用**：简化消息队列的使用，降低开发难度
3. **类型安全**：利用 Rust 的类型系统提供编译时类型检查
4. **高性能**：保持 Rust 的性能优势，最小化抽象层开销
5. **异步支持**：完全支持异步编程模型

## 核心组件

RMS 由以下几个核心组件组成：

### 1. 消息接口 (Message)

消息是 RMS 的基本数据单元，包含消息的主体内容和元数据。

### 2. 消息转换器 (MessageConverter)

负责消息的序列化和反序列化，支持多种格式，如 JSON、Bincode 等。

### 3. 消息通道 (MessageChannel)

定义消息的传输路径，是生产者和消费者之间的桥梁。

### 4. 消息生产者 (MessageProducer)

负责发送消息到消息通道。

### 5. 消息消费者 (MessageConsumer)

负责从消息通道接收消息并处理。

## 支持的消息系统

RMS 设计为支持多种消息系统，目前提供了以下实现：

- **Redis Pub/Sub**：使用 Redis 的发布/订阅功能
- 更多实现正在开发中...

## 使用示例

### 基本用法

```rust
use crate::rms::{
    channel::MessageChannel,
    consumer::MessageConsumer,
    producer::MessageProducer,
};
use crate::rms::examples::redis_pubsub::RedisMessageChannel;
use serde::{Serialize, Deserialize};

// 1. 定义消息结构
#[derive(Debug, Serialize, Deserialize)]
struct MyMessage {
    id: String,
    content: String,
}

// 2. 创建消息通道
let channel = RedisMessageChannel::new("redis://localhost:6379", "my-topic").await?;

// 3. 获取生产者
let producer = channel.create_producer();

// 4. 获取消费者并订阅
let consumer = channel.create_consumer();
consumer.subscribe(|msg: MyMessage| async move {
    println!("收到消息: {:?}", msg);
    Ok(())
}).await?;

// 5. 发送消息
let message = MyMessage {
    id: "1".to_string(),
    content: "Hello RMS!".to_string(),
};
producer.send(message).await?;
```

### 更多示例

请查看 `examples` 目录下的示例代码，了解更多使用场景：

- `redis_pubsub.rs`：Redis Pub/Sub 实现
- `redis_demo.rs`：更完整的 Redis 示例，包含多个主题和消息类型

## 扩展

RMS 设计为可扩展的，您可以轻松实现自己的消息系统适配器：

1. 实现 `MessageChannel` trait
2. 实现 `MessageProducer` trait
3. 实现 `MessageConsumer` trait

## 注意事项

- RMS 需要异步运行时（如 tokio）才能正常工作
- 所有操作都是异步的，需要使用 `async/await`
- 消息类型需要实现 `Serialize` 和 `Deserialize` trait
