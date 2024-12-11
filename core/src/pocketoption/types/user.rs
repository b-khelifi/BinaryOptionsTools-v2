use core::fmt;

use serde::Deserialize;
use serde_json::Value;

use crate::pocketoption::parser::message::WebSocketMessage;

use super::info::MessageInfo;

pub struct UserRequest {
    pub message: Box<WebSocketMessage>,
    pub response_type: MessageInfo,
    pub validator: Box<dyn Fn(&WebSocketMessage) -> bool + Send + Sync>,
    pub sender: tokio::sync::oneshot::Sender<WebSocketMessage>
}

impl fmt::Debug for UserRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message: {:?}", self.message)?;
        write!(f, "Response Type: {:?}", self.response_type)
    }
}

impl<'de> Deserialize<'de> for UserRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let value = Value::deserialize(deserializer)?;
        let message = serde_json::from_value(value.get("message").ok_or(serde::de::Error::missing_field("Missing field 'message'"))?.clone()).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        let response_type = serde_json::from_value(value.get("response_type").ok_or(serde::de::Error::missing_field("Missing field 'response_type'"))?.clone()).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        let (sender, _) = tokio::sync::oneshot::channel::<WebSocketMessage>();
        Ok(Self {
            message,
            response_type,
            validator: Box::new(default_validator),
            sender
        })
    }
}

pub fn default_validator(_validator: &WebSocketMessage) -> bool {
    false
}

impl UserRequest {
    pub fn new(message: WebSocketMessage, response_type: MessageInfo, validator: impl Fn(&WebSocketMessage) -> bool + Send + Sync + 'static) -> (Self, tokio::sync::oneshot::Receiver<WebSocketMessage>) {
        let (sender, reciever) = tokio::sync::oneshot::channel::<WebSocketMessage>();
        let request = Self {
            message: Box::new(message),
            response_type,
            validator: Box::new(validator),
            sender
        };
        (request, reciever)
    }
}

