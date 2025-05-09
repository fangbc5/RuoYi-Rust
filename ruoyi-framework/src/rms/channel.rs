use super::{consumer::MessageConsumer, producer::MessageProducer};

/// 消息通道工厂
pub trait MessageChannelFactory {
    type Producer: MessageProducer;
    type Consumer: MessageConsumer;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// 创建生产者
    fn create_producer(&self, config: &str) -> Result<Self::Producer, Self::Error>;
    
    /// 创建消费者
    fn create_consumer(&self, config: &str) -> Result<Self::Consumer, Self::Error>;
    
    /// 创建双向通道
    fn create_bichannel(&self, config: &str) -> Result<(Self::Producer, Self::Consumer), Self::Error>;
}