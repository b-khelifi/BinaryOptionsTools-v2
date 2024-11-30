use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::tungstenite::Message;

use super::ssid::Ssid;


pub struct WebSocketClient {
    pub ssid: Ssid,
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>

}


#[cfg(test)]
mod tests {
    use std::error::Error;

    use chrono::{format, Utc};
    use tokio_tungstenite::{tungstenite::protocol::Message, connect_async_tls_with_config, tungstenite::{error::TlsError, handshake::client::generate_key, http::{Request, Uri}}, Connector};
    use futures_util::{future, pin_mut, SinkExt, StreamExt};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::pocketoption::{utils::basic::get_index, ws::ssid::Ssid};

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
        
        while let Some(msg) = read.next().await {
            println!("receiving...");
            let msg = match msg.unwrap() {
                Message::Binary(bin) | Message::Ping(bin) | Message::Pong(bin) => format!("Bin: {}", String::from_utf8(bin).unwrap()),
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
                        }
                        _ => {}
                    }
                    
                    text
                },
                Message::Close(close_frame) => String::from("Closed"),
                Message::Frame(frame) => unimplemented!(), 
            };
            println!("Message: {:?}", msg);

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