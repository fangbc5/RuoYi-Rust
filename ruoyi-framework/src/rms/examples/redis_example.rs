use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

use super::redis_impl::RedisMessageChannelFactory;
use crate::rms::channel::MessageChannelFactory;
use crate::rms::consumer::MessageConsumer;
use crate::rms::producer::MessageProducer;
use crate::rms::{Message, MessageHeaders};

// Redis连接地址
const REDIS_URL: &str = "redis://:123456@localhost:6379";

// 定义示例消息类型
#[derive(Debug, Serialize, Deserialize)]
struct ExampleMessage {
    id: String,
    content: String,
}

/// 运行Redis消息示例
pub async fn run_redis_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建消息通道工厂
    let factory = RedisMessageChannelFactory::new(REDIS_URL)?;

    // 创建生产者和消费者
    let (producer, consumer) = factory.create_bichannel("demo")?;

    // 设置消息处理回调
    consumer
        .subscribe("test-topic", |ack_msg| {
            Box::pin(async move {
                let msg = ack_msg.message;
                println!("收到消息: {}", msg.payload);
                println!("消息头: {:?}", msg.headers);

                // 确认消息已成功处理
                (ack_msg.ack)(true);
            })
        })
        .await?;

    // 发送一些测试消息
    for i in 0..5 {
        let mut headers = MessageHeaders::new();
        headers.insert("message-type".to_string(), "test".to_string());
        headers.insert("sequence".to_string(), i.to_string());

        let message = Message {
            payload: format!("这是测试消息 #{}", i),
            headers,
        };

        producer.send("test-topic", message).await?;
        println!("已发送消息 #{}", i);
        sleep(Duration::from_millis(500)).await;
    }

    // 等待所有消息被处理
    sleep(Duration::from_secs(2)).await;

    println!("示例运行完成");
    Ok(())
}

/// 演示批量消息处理
pub async fn run_batch_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建消息通道工厂
    let factory = RedisMessageChannelFactory::new(REDIS_URL)?;

    // 创建生产者和消费者
    let (producer, consumer) = factory.create_bichannel("batch-demo")?;

    // 设置批量消息处理回调
    consumer
        .batch_subscribe("batch-topic", 3, |batch| {
            Box::pin(async move {
                println!("收到批量消息，数量: {}", batch.len());

                for (i, ack_msg) in batch.into_iter().enumerate() {
                    println!("批次消息 #{}: {}", i, ack_msg.message.payload);
                    // 确认消息
                    (ack_msg.ack)(true);
                }
            })
        })
        .await?;

    // 发送一批测试消息
    for i in 0..10 {
        let mut headers = MessageHeaders::new();
        headers.insert("batch-id".to_string(), "test-batch".to_string());

        let message = Message {
            payload: format!("批量测试消息 #{}", i),
            headers,
        };

        producer.send("batch-topic", message).await?;
        println!("已发送批量消息 #{}", i);
        sleep(Duration::from_millis(200)).await;
    }

    // 等待所有消息被处理
    sleep(Duration::from_secs(3)).await;

    println!("批量示例运行完成");
    Ok(())
}
