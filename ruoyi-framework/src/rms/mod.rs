pub mod producer;
pub mod consumer;
pub mod channel;
pub mod converter;
pub mod examples;
pub mod tests;

use std::collections::HashMap;

/// 消息头类型
pub type MessageHeaders = HashMap<String, String>;

/// 通用消息结构
pub struct Message<T> {
    pub payload: T,
    pub headers: MessageHeaders,
}

/// 消息确认回调
pub type AckCallback = Box<dyn FnOnce(bool) + Send>;

/// 带有确认功能的消息
pub struct AcknowledgeableMessage<T> {
    pub message: Message<T>,
    pub ack: AckCallback,
}

#[macro_export]
macro_rules! bind {
    ($factory:expr, $config:expr) => {
        $factory.create_bichannel($config).expect("Failed to create channel")
    };
    (producer $factory:expr, $config:expr) => {
        $factory.create_producer($config).expect("Failed to create producer")
    };
    (consumer $factory:expr, $config:expr) => {
        $factory.create_consumer($config).expect("Failed to create consumer")
    };
}