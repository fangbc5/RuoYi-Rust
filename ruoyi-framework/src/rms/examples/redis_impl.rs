use async_trait::async_trait;
use futures::StreamExt;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::rms::{
    channel::MessageChannelFactory, consumer::MessageConsumer, producer::MessageProducer,
    AckCallback, AcknowledgeableMessage, Message, MessageHeaders,
};

/// Redis消息通道的错误类型
#[derive(Debug, thiserror::Error)]
pub enum RedisMessageError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Channel closed")]
    ChannelClosed,
}

/// Redis消息的格式
#[derive(Serialize, Deserialize)]
struct RedisMessage {
    id: String,
    payload: String,
    headers: MessageHeaders,
}

/// Redis消息通道工厂
pub struct RedisMessageChannelFactory {
    client: Client,
}

impl RedisMessageChannelFactory {
    /// 创建一个新的Redis消息通道工厂
    pub fn new(redis_url: &str) -> Result<Self, RedisMessageError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl MessageChannelFactory for RedisMessageChannelFactory {
    type Producer = RedisMessageProducer;
    type Consumer = RedisMessageConsumer;
    type Error = RedisMessageError;

    fn create_producer(&self, _config: &str) -> Result<Self::Producer, Self::Error> {
        // 返回一个带有 client 的生产者，实际的连接将在首次使用时创建
        Ok(RedisMessageProducer {
            client: self.client.clone(),
        })
    }

    fn create_consumer(&self, _config: &str) -> Result<Self::Consumer, Self::Error> {
        let client = self.client.clone();
        Ok(RedisMessageConsumer {
            client,
            subscribers: Arc::new(Mutex::new(Vec::new())),
        })
    }

    fn create_bichannel(
        &self,
        config: &str,
    ) -> Result<(Self::Producer, Self::Consumer), Self::Error> {
        Ok((self.create_producer(config)?, self.create_consumer(config)?))
    }
}

/// Redis消息生产者
pub struct RedisMessageProducer {
    client: Client,
}

#[async_trait]
impl MessageProducer for RedisMessageProducer {
    type Payload = String;
    type Error = RedisMessageError;

    async fn send(
        &self,
        destination: &str,
        message: Message<Self::Payload>,
    ) -> Result<(), Self::Error> {
        let redis_message = RedisMessage {
            id: Uuid::new_v4().to_string(),
            payload: message.payload,
            headers: message.headers,
        };

        // 每次发送时创建一个新的连接
        let mut conn = self.client.get_async_connection().await?;
        let serialized = serde_json::to_string(&redis_message)?;
        conn.publish::<_, _, ()>(destination, serialized).await?;
        Ok(())
    }

    async fn send_and_wait(
        &self,
        destination: &str,
        message: Message<Self::Payload>,
    ) -> Result<(), Self::Error> {
        // 对于Redis实现，我们简单地发送消息
        // 实际上可以添加确认机制，但这需要更复杂的实现
        self.send(destination, message).await
    }
}

/// Redis消息消费者
pub struct RedisMessageConsumer {
    client: Client,
    subscribers: Arc<Mutex<Vec<(String, mpsc::Sender<String>)>>>,
}

#[async_trait]
impl MessageConsumer for RedisMessageConsumer {
    type Payload = String;
    type Error = RedisMessageError;

    async fn subscribe<F>(&self, destination: &str, callback: F) -> Result<(), Self::Error>
    where
        F: Fn(AcknowledgeableMessage<Self::Payload>) -> Pin<Box<dyn Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let (tx, mut rx) = mpsc::channel(100);

        // 保存订阅信息
        {
            let mut subscribers = self.subscribers.lock().await;
            subscribers.push((destination.to_string(), tx));
        }

        // 创建Redis PubSub连接
        let mut pubsub = self.client.get_async_connection().await?.into_pubsub();
        pubsub.subscribe(destination).await?;

        // 启动监听任务
        let callback = Arc::new(callback);

        tokio::spawn(async move {
            let mut pubsub_stream = pubsub.on_message();

            loop {
                tokio::select! {
                    Some(msg) = pubsub_stream.next() => {
                        if let Ok(payload) = msg.get_payload::<String>() {
                            if let Ok(redis_message) = serde_json::from_str::<RedisMessage>(&payload) {
                                let message = Message {
                                    payload: redis_message.payload,
                                    headers: redis_message.headers,
                                };

                                let ack_callback: AckCallback = Box::new(|success| {
                                    // 在实际应用中，这里可以记录确认状态
                                    if success {
                                        println!("Message processed successfully");
                                    } else {
                                        println!("Message processing failed");
                                    }
                                });

                                let ack_message = AcknowledgeableMessage {
                                    message,
                                    ack: ack_callback,
                                };

                                callback(ack_message).await;
                            }
                        }
                    },
                    _ = rx.recv() => {
                        // 接收到关闭信号，退出循环
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn batch_subscribe<F>(
        &self,
        destination: &str,
        batch_size: usize,
        callback: F,
    ) -> Result<(), Self::Error>
    where
        F: Fn(
                Vec<AcknowledgeableMessage<Self::Payload>>,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let (tx, mut rx) = mpsc::channel(100);

        // 保存订阅信息
        {
            let mut subscribers = self.subscribers.lock().await;
            subscribers.push((destination.to_string(), tx));
        }

        // 创建Redis PubSub连接
        let mut pubsub = self.client.get_async_connection().await?.into_pubsub();
        pubsub.subscribe(destination).await?;

        // 启动监听任务
        let callback = Arc::new(callback);

        tokio::spawn(async move {
            let mut pubsub_stream = pubsub.on_message();
            let mut batch = Vec::with_capacity(batch_size);
            let mut should_exit = false;

            loop {
                tokio::select! {
                    Some(msg) = pubsub_stream.next(), if !should_exit => {
                        if let Ok(payload) = msg.get_payload::<String>() {
                            if let Ok(redis_message) = serde_json::from_str::<RedisMessage>(&payload) {
                                let message = Message {
                                    payload: redis_message.payload,
                                    headers: redis_message.headers,
                                };

                                let ack_callback: AckCallback = Box::new(|success| {
                                    if success {
                                        println!("Message processed successfully");
                                    } else {
                                        println!("Message processing failed");
                                    }
                                });

                                let ack_message = AcknowledgeableMessage {
                                    message,
                                    ack: ack_callback,
                                };

                                batch.push(ack_message);

                                if batch.len() >= batch_size {
                                    // 处理批次
                                    let batch_to_process = std::mem::replace(&mut batch, Vec::with_capacity(batch_size));
                                    callback(batch_to_process).await;
                                }
                            }
                        }
                    },
                    _ = rx.recv() => {
                        // 接收到关闭信号，标记退出但继续处理已接收的消息
                        should_exit = true;

                        // 处理剩余的消息并退出
                        if !batch.is_empty() {
                            callback(batch).await;
                            batch = Vec::new();
                        }

                        // 如果不需要处理更多消息了，就退出循环
                        if should_exit && batch.is_empty() {
                            break;
                        }
                    }
                }

                // 如果标记了退出且没有未处理的消息，则退出循环
                if should_exit && batch.is_empty() {
                    break;
                }
            }
        });

        Ok(())
    }
}

// 为Redis消息消费者实现Drop trait，以便在消费者被丢弃时取消订阅
impl Drop for RedisMessageConsumer {
    fn drop(&mut self) {
        // 这里不能直接运行异步代码，所以我们只是提供一个提示
        println!("RedisMessageConsumer dropped, subscriptions will be cleaned up");
    }
}
