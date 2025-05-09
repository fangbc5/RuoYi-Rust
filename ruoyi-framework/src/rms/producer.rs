use super::Message;


/// 消息生产者 trait
#[async_trait::async_trait]
pub trait MessageProducer {
    type Payload;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// 发送消息
    async fn send(&self, destination: &str, message: Message<Self::Payload>) -> Result<(), Self::Error>;
    
    /// 发送消息并等待确认
    async fn send_and_wait(&self, destination: &str, message: Message<Self::Payload>) -> Result<(), Self::Error>;
}