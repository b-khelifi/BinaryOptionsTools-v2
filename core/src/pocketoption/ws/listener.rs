use tokio::sync::mpsc::Sender;

use async_trait::async_trait;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use crate::pocketoption::{error::{PocketOptionError, PocketResult}, parser::message::{self, WebSocketMessage}, types::info::MessageInfo};

use super::{basic::WebSocketClient, ssid::Ssid};

#[async_trait]
pub trait EventListener: Clone + Send + Sync + 'static {
    fn on_raw_message(&self, message: Message) -> PocketResult<Message> {
        Ok(message)
    }

    fn on_message(&self, message: WebSocketMessage) -> PocketResult<WebSocketMessage> {
        Ok(message)
    }

    async fn process_message(&self, message: &Message, previous: &MessageInfo, sender: &Sender<Message>) -> PocketResult<(Option<MessageInfo>, bool)> {
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
pub struct Handler {
    ssid: Ssid
}

impl Handler {
    pub fn new(ssid: Ssid) -> Self {
        Self { ssid }
    }

    pub fn handle_binary_msg(&self, bytes: &Vec<u8>, previous: &MessageInfo) -> PocketResult<WebSocketMessage> {
        let msg = String::from_utf8(bytes.to_owned())?;
        let message = WebSocketMessage::parse_with_context(msg, previous)?;
        Ok(message)
    }

    pub async fn handle_text_msg(&self, text: &str, sender: &Sender<Message>) -> PocketResult<Option<MessageInfo>> {
        match text {
            _ if text.starts_with('0') && text.contains("sid") => {
                sender.send(Message::Text("40".into())).await?;
            },
            _ if text.starts_with("40") && text.contains("sid") => {
                sender.send(Message::Text(self.ssid.to_string())).await?;
            },
            _ if text == "2" => {
                sender.send(Message::Text("3".into())).await?;
                // write.send(Message::Text("3".into())).await.unwrap();
                // write.flush().await.unwrap();
            },
            _ if text.starts_with("451-") => {
                let msg = text.strip_prefix("451-").unwrap();
                let (info, _): (MessageInfo, Value) = serde_json::from_str(msg)?;
                return Ok(Some(info));
            }
            _ => {}
        }
        
        Ok(None)
    }
}

#[async_trait::async_trait]
impl EventListener for Handler {
    async fn process_message(&self, message: &Message, previous: &MessageInfo, sender: &Sender<Message>) -> PocketResult<(Option<MessageInfo> ,bool)> {
        match message {
            Message::Binary(binary) => {self.handle_binary_msg(binary, previous)?;},
            Message::Text(text) => {
                let res = self.handle_text_msg(text, sender).await?;
                println!("{:?}", res);
                return Ok((res, false))
            },
            Message::Frame(frame) => {},
            Message::Ping(binary) => {},
            Message::Pong(binary) => {},
            Message::Close(close) => return Ok((None, true)),
        } 
        Ok((None, false))
    }   
}