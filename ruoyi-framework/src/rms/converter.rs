use super::Message;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// 消息转换器 trait
pub trait MessageConverter {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    fn convert(&self, message: Message<Self::Input>) -> Result<Message<Self::Output>, Self::Error>;
}

pub struct JsonConverter<T, U> {
    _input: PhantomData<T>,
    _output: PhantomData<U>,
}

impl<T: Serialize, U: for<'de> Deserialize<'de>> MessageConverter for JsonConverter<T, U> {
    type Input = T;
    type Output = U;
    type Error = serde_json::Error;

    fn convert(&self, message: Message<T>) -> Result<Message<U>, serde_json::Error> {
        let json = serde_json::to_string(&message.payload)?;
        let output: U = serde_json::from_str(&json)?;
        Ok(Message {
            payload: output,
            headers: message.headers,
        })
    }
}
