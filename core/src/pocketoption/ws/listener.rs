use async_channel::Sender;

use async_trait::async_trait;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

use crate::{
    error::BinaryOptionsResult,
    general::{traits::MessageHandler, types::MessageType},
    pocketoption::{
        error::PocketResult,
        parser::message::WebSocketMessage,
        types::{base::ChangeSymbol, data::Data, info::MessageInfo},
    },
};

use super::ssid::Ssid;

#[async_trait]
pub trait EventListener: Clone + Send + Sync + 'static {
    fn on_raw_message(&self, message: Message) -> PocketResult<Message> {
        Ok(message)
    }

    fn on_message(&self, message: WebSocketMessage) -> PocketResult<WebSocketMessage> {
        Ok(message)
    }

    async fn process_message(
        &self,
        _message: &Message,
        _previous: &MessageInfo,
        _sender: &Sender<Message>,
        _local_sender: &Sender<WebSocketMessage>,
        _data: &Data,
    ) -> PocketResult<(Option<MessageInfo>, bool)> {
        Ok((None, false))
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
    ssid: Ssid,
}

impl Handler {
    pub fn new(ssid: Ssid) -> Self {
        Self { ssid }
    }

    pub fn handle_binary_msg(
        &self,
        bytes: &Vec<u8>,
        previous: &Option<MessageInfo>,
    ) -> PocketResult<WebSocketMessage> {
        let msg = String::from_utf8(bytes.to_owned())?;
        let message = match previous {
            Some(previous) => WebSocketMessage::parse_with_context(msg, previous)?,
            None => {
                let message: WebSocketMessage = serde_json::from_str(&msg)?;
                message
            }
        };

        Ok(message)
    }
    pub fn temp_bin(
        &self,
        bytes: &Vec<u8>,
        previous: &MessageInfo,
    ) -> PocketResult<WebSocketMessage> {
        let msg = String::from_utf8(bytes.to_owned())?;
        WebSocketMessage::parse_with_context(msg, previous)
    }

    pub async fn handle_text_msg(
        &self,
        text: &str,
        sender: &Sender<Message>,
    ) -> PocketResult<Option<MessageInfo>> {
        match text {
            _ if text.starts_with('0') && text.contains("sid") => {
                sender.send(Message::text("40")).await?;
            }
            _ if text.starts_with("40") && text.contains("sid") => {
                sender.send(Message::text(self.ssid.to_string())).await?;
            }
            _ if text == "2" => {
                sender.send(Message::text("3")).await?;
                // write.send(Message::text("3".into())).await.unwrap();
                // write.flush().await.unwrap();
            }
            _ if text.starts_with("451-") => {
                let msg = text.strip_prefix("451-").unwrap();
                let (info, _): (MessageInfo, Value) = serde_json::from_str(msg)?;
                if info == MessageInfo::UpdateClosedDeals {
                    sender
                        .send(Message::text(
                            WebSocketMessage::ChangeSymbol(ChangeSymbol {
                                asset: "AUDNZD_otc".into(),
                                period: 60,
                            })
                            .to_string(),
                        ))
                        .await?;
                }
                return Ok(Some(info));
            }
            _ => {}
        }

        Ok(None)
    }
}

#[async_trait::async_trait]
impl EventListener for Handler {
    async fn process_message(
        &self,
        message: &Message,
        previous: &MessageInfo,
        sender: &Sender<Message>,
        local_sender: &Sender<WebSocketMessage>,
        data: &Data,
    ) -> PocketResult<(Option<MessageInfo>, bool)> {
        match message {
            Message::Binary(binary) => {
                let msg = self.temp_bin(&binary.to_vec(), previous)?;
                if let WebSocketMessage::UpdateStream(stream) = &msg {
                    match stream.0.first() {
                        Some(item) => data.update_server_time(item.time.timestamp()).await,
                        None => warn!("Missing data in 'updateStream' message"),
                    }
                }
                if let Some(senders) = data.get_request(&msg).await? {
                    for s in senders {
                        s.send(msg.clone())?;
                    }
                }
                local_sender.send(msg).await?;
            }
            Message::Text(text) => {
                let res = self.handle_text_msg(&text.to_string(), sender).await?;
                return Ok((res, false));
            }
            Message::Frame(_) => {} // TODO:
            Message::Ping(_) => {}  // TODO:
            Message::Pong(_) => {}  // TODO:
            Message::Close(_) => return Ok((None, true)),
        }
        Ok((None, false))
    }
}

#[async_trait]
impl MessageHandler for Handler {
    type Transfer = WebSocketMessage;

    async fn process_message(
        &self,
        message: &Message,
        previous: &Option<MessageInfo>,
        sender: &Sender<Message>,
    ) -> BinaryOptionsResult<(Option<MessageType<WebSocketMessage>>, bool)> {
        match message {
            Message::Binary(binary) => {
                let msg = self.handle_binary_msg(&binary.to_vec(), previous)?;
                return Ok((Some(MessageType::Transfer(msg)), false));
            }
            Message::Text(text) => {
                let res = self.handle_text_msg(&text.to_string(), sender).await?;
                return Ok((res.map(MessageType::Info), false));
            }
            Message::Frame(_) => {} // TODO:
            Message::Ping(_) => {}  // TODO:
            Message::Pong(_) => {}  // TODO:
            Message::Close(_) => return Ok((None, true)),
        }
        Ok((None, false))
    }
}
