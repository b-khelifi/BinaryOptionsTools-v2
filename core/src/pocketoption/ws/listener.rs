use async_trait::async_trait;
use tokio_tungstenite::tungstenite::Message;

use crate::pocketoption::{error::{PocketOptionError, PocketResult}, parser::message::{self, WebSocketMessage}, types::info::MessageInfo};

use super::basic::WebSocketClient;

#[async_trait]
pub trait EventListener: Clone + Send + Sync + 'static {
    fn on_raw_message(&self, message: Message) -> PocketResult<Message> {
        Ok(message)
    }

    fn on_message(&self, message: WebSocketMessage) -> PocketResult<WebSocketMessage> {
        Ok(message)
    }

    fn process_message(&self, message: &Message, previous: &MessageInfo) -> PocketResult<(Option<WebSocketMessage>, bool)> {
        Ok((None,false))
    }

    async fn on_raw_message_async(&self, message: Message) -> PocketResult<Message> {
        Ok(message)
    }

    async fn on_message_async(&self, message: WebSocketMessage) -> PocketResult<WebSocketMessage> {
        Ok(message)
    }
}


#[derive(Clone)]
pub struct Handler;

impl Handler {
    pub fn handle_binary_msg(&self, bytes: &Vec<u8>, previous: &MessageInfo) -> PocketResult<WebSocketMessage> {
        let msg = String::from_utf8(bytes.to_owned())?;
        let message = WebSocketMessage::parse_with_context(msg, previous)?;
        if let WebSocketMessage::UpdateAssets(_) = &message {
            dbg!("Recieved update assets");
        } else if let WebSocketMessage::UpdateHistoryNew(_) = &message {
            dbg!("Recieved update history new");

        } else {
            dbg!("Binary Message: ", &message);
        }
        Ok(message)
    }
}

impl EventListener for Handler {
    fn process_message(&self ,message: &Message, previous: &MessageInfo) -> PocketResult<(Option<WebSocketMessage> ,bool)> {
        match message {
            Message::Binary(binary) => {self.handle_binary_msg(&binary, previous)?;},
            Message::Text(text) => {

            },
            Message::Frame(frame) => {},
            Message::Ping(binary) => {},
            Message::Pong(binary) => {},
            Message::Close(close) => return Ok((None, true)),
        } 
        Ok((None, false))
    }
}