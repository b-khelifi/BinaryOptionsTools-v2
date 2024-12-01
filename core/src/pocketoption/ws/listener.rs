use async_trait::async_trait;
use tokio_tungstenite::tungstenite::Message;

use crate::pocketoption::{error::PocketOptionError, parser::message::{self, WebSocketMessage}};

use super::basic::WebSocketClient;

#[async_trait]
pub trait EventListener {
    fn on_raw_message(&self, message: Message) -> Result<Message, PocketOptionError> {
        Ok(message)
    }

    fn on_message(&self, message: WebSocketMessage) -> Result<WebSocketMessage, PocketOptionError> {
        Ok(message)
    }

    async fn on_raw_message_async(&self, message: Message) -> Result<Message, PocketOptionError> {
        Ok(message)
    }

    async fn on_message_async(&self, message: WebSocketMessage) -> Result<WebSocketMessage, PocketOptionError> {
        Ok(message)
    }
}
