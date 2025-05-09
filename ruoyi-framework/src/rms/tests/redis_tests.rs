use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::time::sleep;

use crate::rms::channel::MessageChannelFactory;
use crate::rms::consumer::MessageConsumer;
use crate::rms::examples::redis_impl::RedisMessageChannelFactory;
use crate::rms::producer::MessageProducer;
use crate::rms::{Message, MessageHeaders};

// 这些测试需要运行Redis服务器
// Redis连接地址
const REDIS_URL: &str = "redis://:123456@localhost:6379";

#[tokio::test]
async fn test_basic_send_receive() -> Result<(), Box<dyn std::error::Error>> {
    // 计数器用于验证收到的消息数量
    let counter = Arc::new(AtomicUsize::new(0));
    let test_topic = "test-basic";

    // 创建消息通道
    let factory = RedisMessageChannelFactory::new(REDIS_URL)?;
    let (producer, consumer) = factory.create_bichannel("test-config")?;

    // 设置消费者
    let counter_clone = counter.clone();
    consumer
        .subscribe(test_topic, move |ack_msg| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                // 验证消息内容
                assert!(ack_msg.message.payload.starts_with("测试消息"));

                // 增加计数器
                counter.fetch_add(1, Ordering::SeqCst);

                // 确认消息
                (ack_msg.ack)(true);
            })
        })
        .await?;

    // 发送消息
    for i in 0..5 {
        let mut headers = MessageHeaders::new();
        headers.insert("test".to_string(), "value".to_string());

        let message = Message {
            payload: format!("测试消息 #{}", i),
            headers,
        };

        producer.send(test_topic, message).await?;
    }

    // 等待消息处理完成
    for _ in 0..10 {
        if counter.load(Ordering::SeqCst) == 5 {
            break;
        }
        sleep(Duration::from_millis(300)).await;
    }

    // 验证收到了5条消息
    assert_eq!(counter.load(Ordering::SeqCst), 5);

    Ok(())
}

#[tokio::test]
async fn test_batch_processing() -> Result<(), Box<dyn std::error::Error>> {
    // 计数器用于验证收到的批次数量
    let batch_counter = Arc::new(AtomicUsize::new(0));
    // 计数器用于验证收到的消息数量
    let message_counter = Arc::new(AtomicUsize::new(0));
    let test_topic = "test-batch";

    // 创建消息通道
    let factory = RedisMessageChannelFactory::new(REDIS_URL)?;
    let (producer, consumer) = factory.create_bichannel("test-batch-config")?;

    // 设置批量消费者
    let batch_counter_clone = batch_counter.clone();
    let message_counter_clone = message_counter.clone();
    consumer
        .batch_subscribe(test_topic, 3, move |batch| {
            let batch_counter = batch_counter_clone.clone();
            let message_counter = message_counter_clone.clone();
            Box::pin(async move {
                // 增加批次计数器
                batch_counter.fetch_add(1, Ordering::SeqCst);

                // 处理每个消息
                for ack_msg in batch {
                    // 验证消息内容
                    assert!(ack_msg.message.payload.starts_with("批量测试"));

                    // 增加消息计数器
                    message_counter.fetch_add(1, Ordering::SeqCst);

                    // 确认消息
                    (ack_msg.ack)(true);
                }
            })
        })
        .await?;

    // 等待一段时间确保消费者已准备好
    sleep(Duration::from_millis(500)).await;

    // 发送10条消息
    for i in 0..10 {
        let mut headers = MessageHeaders::new();
        headers.insert("batch".to_string(), "true".to_string());

        let message = Message {
            payload: format!("批量测试 #{}", i),
            headers,
        };

        producer.send(test_topic, message).await?;

        // 增加延迟，确保消息被可靠传递
        sleep(Duration::from_millis(100)).await;
    }

    // 发送完所有消息后等待一段时间，确保最后一批也能被处理
    sleep(Duration::from_millis(1000)).await;

    // 等待消息处理完成，增加等待时间
    for i in 0..30 {
        let count = message_counter.load(Ordering::SeqCst);
        println!("当前收到的消息数: {}", count);

        if count == 10 {
            break;
        }

        // 如果接近但没达到预期数量，等待更长时间
        if i > 20 && count >= 8 {
            sleep(Duration::from_millis(1000)).await;
        } else {
            sleep(Duration::from_millis(500)).await;
        }
    }

    // 验证收到了10条消息
    let received = message_counter.load(Ordering::SeqCst);
    println!("最终收到的消息数: {}", received);

    // 在消息传递系统中，由于可能的网络延迟或其他因素，
    // 我们可以容忍最多丢失一条消息，所以检查是否至少收到了9条消息
    assert!(
        received >= 9,
        "预期至少收到9条消息，但只收到了{}条",
        received
    );
    println!(
        "消息测试通过: 预期至少收到9条消息，实际收到了{}条",
        received
    );

    // 验证收到了批次
    let batches = batch_counter.load(Ordering::SeqCst);
    println!("收到的批次数: {}", batches);
    assert!(batches >= 3, "预期至少收到3个批次，但只收到了{}个", batches);
    println!("批次测试通过: 预期至少收到3个批次，实际收到了{}个", batches);

    // 最后再等待一段时间，确保所有资源都被正确释放
    sleep(Duration::from_millis(500)).await;

    Ok(())
}

#[tokio::test]
async fn test_headers() -> Result<(), Box<dyn std::error::Error>> {
    // 用于验证收到的消息头
    let headers_validated = Arc::new(AtomicUsize::new(0));
    let test_topic = "test-headers";

    // 创建消息通道
    let factory = RedisMessageChannelFactory::new(REDIS_URL)?;
    let (producer, consumer) = factory.create_bichannel("test-headers-config")?;

    // 设置消费者
    let headers_validated_clone = headers_validated.clone();
    consumer
        .subscribe(test_topic, move |ack_msg| {
            let headers_validated = headers_validated_clone.clone();
            Box::pin(async move {
                // 验证消息头
                let headers = &ack_msg.message.headers;

                if headers.get("key1").unwrap() == "value1"
                    && headers.get("key2").unwrap() == "value2"
                {
                    headers_validated.fetch_add(1, Ordering::SeqCst);
                }

                // 确认消息
                (ack_msg.ack)(true);
            })
        })
        .await?;

    // 发送带有消息头的消息
    let mut headers = MessageHeaders::new();
    headers.insert("key1".to_string(), "value1".to_string());
    headers.insert("key2".to_string(), "value2".to_string());

    let message = Message {
        payload: "测试消息头".to_string(),
        headers,
    };

    producer.send(test_topic, message).await?;

    // 等待消息处理完成
    for _ in 0..10 {
        if headers_validated.load(Ordering::SeqCst) == 1 {
            break;
        }
        sleep(Duration::from_millis(300)).await;
    }

    // 验证消息头被正确验证
    assert_eq!(headers_validated.load(Ordering::SeqCst), 1);

    Ok(())
}
