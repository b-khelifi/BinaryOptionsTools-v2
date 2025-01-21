use async_trait::async_trait;
use serde_json::Value;

use binary_options_tools_core::{
    error::{BinaryOptionsResult, BinaryOptionsToolsError},
    general::{send::SenderMessage, traits::MessageHandler, types::MessageType},
    reimports::Message,
};

use crate::pocketoption::{
    error::PocketResult,
    parser::message::WebSocketMessage,
    types::{base::ChangeSymbol, info::MessageInfo},
};

use super::ssid::Ssid;

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
        sender: &SenderMessage,
    ) -> BinaryOptionsResult<Option<MessageInfo>> {
        match text {
            _ if text.starts_with('0') && text.contains("sid") => {
                sender.priority_send(Message::text("40")).await?;
            }
            _ if text.starts_with("40") && text.contains("sid") => {
                sender
                    .priority_send(Message::text(self.ssid.to_string()))
                    .await?;
            }
            _ if text == "2" => {
                sender.priority_send(Message::text("3")).await?;
                // write.send(Message::text("3".into())).await.unwrap();
                // write.flush().await.unwrap();
            }
            _ if text.starts_with("451-") => {
                let msg = text.strip_prefix("451-").unwrap();
                let (info, _): (MessageInfo, Value) =
                    serde_json::from_str(msg).map_err(BinaryOptionsToolsError::from)?;
                if info == MessageInfo::UpdateClosedDeals {
                    sender
                        .priority_send(Message::text(
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

#[async_trait]
impl MessageHandler for Handler {
    type Transfer = WebSocketMessage;

    async fn process_message(
        &self,
        message: &Message,
        previous: &Option<MessageInfo>,
        sender: &SenderMessage,
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
