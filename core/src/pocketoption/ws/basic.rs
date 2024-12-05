use std::sync::Arc;

use tokio::{net::TcpStream, sync::{mpsc::{Receiver, Sender}, Mutex}};
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::{handshake::client::generate_key, http::Request, Message}, Connector, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::pocketoption::{error::{PocketOptionError, PocketResult}, parser::message::WebSocketMessage, types::{info::MessageInfo, update::UpdateBalance}};

use super::ssid::Ssid;


pub struct WebSocketClient {
    pub ssid: Ssid,
    pub ws: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>
    // pub sender: Sender<Message>,
    // pub receiver: Receiver<Message>,
    // pub balance: UpdateBalance

}

impl WebSocketClient {
    pub async fn connect(&self) -> PocketResult<()> {
        let tls_connector = native_tls::TlsConnector::builder()
        .build()
        .unwrap();

        let connector = Connector::NativeTls(tls_connector);

        let url = self.ssid.server().await?;
        let user_agent = self.ssid.user_agent();
        let t_url = Url::parse(&url).map_err(|e| PocketOptionError::GeneralParsingError(format!("Error getting host, {e}")))?;
        let host = t_url.host_str().ok_or(PocketOptionError::GeneralParsingError("Host not found".into()))?;
        let request = Request::builder().uri(url)
            .header("Origin", "https://pocketoption.com")
            .header("Cache-Control", "no-cache")
            .header("User-Agent", user_agent)
            .header("Upgrade", "websocket")
            .header("Connection", "upgrade")
            .header("Sec-Websocket-Key", generate_key())
            .header("Sec-Websocket-Version", "13")
            .header("Host", host)

            .body(())?;

        let (ws, _) = connect_async_tls_with_config(request, None, false, Some(connector)).await?;


        Ok(())
    }

    pub fn handle_binary_msg(&self, bytes: Vec<u8>, previous: MessageInfo) -> PocketResult<WebSocketMessage> {
        let msg = String::from_utf8(bytes)?;
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

    pub fn process_message(&self, message: Message, previous: MessageInfo) -> PocketResult<Option<MessageInfo>> {
        match message {
            Message::Binary(binary) => {self.handle_binary_msg(binary, previous)?;},
            Message::Text(text) => {

            },
            Message::Frame(frame) => {},
            Message::Ping(binary) => {},
            Message::Pong(binary) => {},
            Message::Close(close) => {},
        } 
        Ok(None)

    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use chrono::{format, Utc};
    use serde_json::Value;
    use tokio_tungstenite::{tungstenite::protocol::Message, connect_async_tls_with_config, tungstenite::{error::TlsError, handshake::client::generate_key, http::{Request, Uri}}, Connector};
    use futures_util::{future, pin_mut, SinkExt, StreamExt};
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex};

    use crate::pocketoption::{parser::message::WebSocketMessage, types::info::MessageInfo, utils::basic::get_index, ws::{basic::WebSocketClient, ssid::Ssid}};

    use crate::pocketoption::parser::basic::LoadHistoryPeriod;
    
    fn get_candles() -> Result<String, Box<dyn Error>> {
        let time = Utc::now().timestamp();
        let period = 60;
        let offset = 900;
        let history_period = LoadHistoryPeriod {
            active: "AUDNZD_otc".into(),
            period,
            time,
            index: get_index()?,
            offset
        };
        Ok(serde_json::to_string(&history_period)?)
    }

    #[tokio::test]
    async fn test_connect() -> Result<(), Box<dyn Error>> {
        let tls_connector = native_tls::TlsConnector::builder()
        .build()
        .unwrap();

        let connector = Connector::NativeTls(tls_connector);
        let ssid: Ssid = Ssid::parse(r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#)?;

        let client = WebSocketClient { ssid: ssid.clone(), ws: Arc::new(Mutex::new(None)) };
        let url = url::Url::parse("wss://demo-api-eu.po.market/socket.io/?EIO=4&transport=websocket")?;
        let host = url.host_str().unwrap();
        let request = Request::builder().uri(url.to_string())
            .header("Origin", "https://pocketoption.com")
            .header("Cache-Control", "no-cache")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .header("Upgrade", "websocket")
            .header("Connection", "upgrade")
            .header("Sec-Websocket-Key", generate_key())
            .header("Sec-Websocket-Version", "13")
            .header("Host", host)

            .body(())?;
        let (ws, _) = connect_async_tls_with_config(request, None, false, Some(connector)).await?;
        let (mut write, mut read) = ws.split();

        println!("sending");
        let msg = format!( "[loadHistoryPeriod, {}]", get_candles()?);
        dbg!(&msg);
        // write.send(Message::Text(msg)).await?;
        // write.flush().await?;
        println!("sent");
        let mut lop = 0;

        while let Some(msg) = read.next().await {
            lop += 1;
            if lop % 5 == 0 {
                write.send(Message::Text(r#"42["changeSymbol",{"asset":"EURTRY_otc","period":3600}]"#.into())).await.unwrap();
                write.flush().await.unwrap();

            }
            println!("receiving...");
            let message = msg.unwrap();
            client.process_message(message.clone(), MessageInfo::None);
            let msg = match message {
                Message::Binary(bin) | Message::Ping(bin) | Message::Pong(bin) => {
                    let msg = String::from_utf8(bin).unwrap();
                    let parsed = WebSocketMessage::parse(&msg);
                    // dbg!(parsed);
                    format!("Bin: {}", &msg)
                },
                Message::Text(text) => {
                    let base = text.clone();
                    match base {
                        _ if base.starts_with('0') && base.contains("sid") => {
                            write.send(Message::Text("40".into())).await.unwrap();
                            write.flush().await.unwrap();
                        },
                        _ if base.starts_with("40") && base.contains("sid") => {
                            write.send(Message::Text(ssid.to_string())).await.unwrap();
                            write.flush().await.unwrap();
                        },
                        _ if base == "2" => {
                            write.send(Message::Text("3".into())).await.unwrap();
                            write.flush().await.unwrap();
                        },
                        _ if base.starts_with("451-") => {
                            let msg = base.strip_prefix("451-").unwrap();
                            let (info, data): (MessageInfo, Value) = serde_json::from_str(msg)?;
                            println!("Recieved message: {}", info)
                        }
                        _ => {}
                    }
                    
                    text
                },
                Message::Close(close_frame) => String::from("Closed"),
                Message::Frame(frame) => unimplemented!(), 
            };

        }
    
    
        Ok(())
    }

    #[test]
    fn test_bytes() -> Result<(), Box<dyn Error>> {
        let bits = vec![77, 105, 115, 115, 105, 110, 103, 32, 111, 114, 32, 105, 110, 118, 97, 108, 105, 100, 32, 83, 101, 99, 45, 87, 101, 98, 83, 111, 99, 107, 101, 116, 45, 75, 101, 121, 32, 104, 101, 97, 100, 101, 114];
        let string = String::from_utf8(bits)?;
        dbg!(string);
        Ok(())
    }
}