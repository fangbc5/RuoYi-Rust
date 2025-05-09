use std::{future::Future, pin::Pin};

use super::AcknowledgeableMessage;

/// 消息消费者 trait
#[async_trait::async_trait]
pub trait MessageConsumer {
    type Payload;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// 订阅消息
    async fn subscribe<F>(&self, destination: &str, callback: F) -> Result<(), Self::Error>
    where
        F: Fn(AcknowledgeableMessage<Self::Payload>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static;
    
    /// 批量订阅消息
    async fn batch_subscribe<F>(&self, destination: &str, batch_size: usize, callback: F) -> Result<(), Self::Error>
    where
        F: Fn(Vec<AcknowledgeableMessage<Self::Payload>>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static;
}