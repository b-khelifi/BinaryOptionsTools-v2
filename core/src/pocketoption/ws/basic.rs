use std::sync::Arc;

use futures_util::{future::{try_join, try_join3}, stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::{mpsc::{Receiver, Sender}, Mutex}, task::JoinHandle};
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::{handshake::client::generate_key, http::Request, Message}, Connector, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};
use url::Url;
use uuid::Uuid;

use crate::pocketoption::{error::{PocketOptionError, PocketResult}, parser::message::WebSocketMessage, types::{data::Data, info::MessageInfo, update::UpdateBalance}};

use super::{listener::{EventListener, Handler}, ssid::Ssid};

const MAX_ALLOWED_LOOPS: u32 = 8;

pub struct WebSocketClient<T: EventListener> {
    pub ssid: Ssid,
    pub demo: bool,
    pub handler: T,
    // pub balance: UpdateBalance
    pub data: Data,
    sender: Sender<WebSocketMessage>,
    event_loop: JoinHandle<()>

}

impl<T: EventListener> WebSocketClient<T> {
    pub async fn new(ssid: impl ToString, demo: bool) -> PocketResult<WebSocketClient<Handler>> {
        let handler = Handler::new(Ssid::parse(ssid.to_string().clone())?);
        WebSocketClient::init(ssid, demo, handler).await
    }

    pub async fn init(ssid: impl ToString, demo: bool, handler: T) -> PocketResult<Self> {
        let ssid = Ssid::parse(ssid)?;
        let _connection = Self::connect(ssid.clone(), demo).await?;
        let data = Data::default();
        let (event_loop, sender) = Self::start_loops(handler.clone(), ssid.clone(), demo, data.clone()).await?;
        println!("Initialized");
        Ok(Self {
            ssid,
            demo,
            handler,
            data,
            sender,
            event_loop
        })
    }   

    pub async fn buy(&self, asset: String, amount: f64) -> PocketResult<Uuid> {
        
        todo!()
    }

    pub async fn connect(ssid: Ssid, demo: bool) -> PocketResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let tls_connector = native_tls::TlsConnector::builder()
            .build()?;

        let connector = Connector::NativeTls(tls_connector);

        let url = ssid.server(demo).await?;
        let user_agent = ssid.user_agent();
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
        println!("Connected!");

        Ok(ws)
    }

    async fn start_loops(handler: T, ssid: Ssid, demo: bool, data: Data) -> PocketResult<(JoinHandle<()>, Sender<WebSocketMessage>)> {

        let (mut write, mut read) = WebSocketClient::<T>::connect(ssid.clone(), demo).await?.split();
        let (sender, mut reciever) = tokio::sync::mpsc::channel(128);
        let (msg_sender, mut msg_reciever) = tokio::sync::mpsc::channel(128);
        let sender_msg = msg_sender.clone();
        let task = tokio::task::spawn(async move {
            let previous = MessageInfo::None;
            let mut loops = 0;
            loop {

                let listener_future = WebSocketClient::<T>::listener_loop(data.clone(), handler.clone(), previous.clone(), &mut read, &sender, &sender_msg);
                let sender_future = WebSocketClient::<T>::sender_loop(&mut write, &mut reciever);
                let update_loop = WebSocketClient::<T>::update_loop(data.clone(), &mut msg_reciever, &sender);
                match try_join3(listener_future, sender_future, update_loop).await {
                    Ok(_) => {
                        if let Ok(websocket) = WebSocketClient::<T>::connect(ssid.clone(), demo).await {
                            (write, read) = websocket.split();
                        } else {
                            loops += 1;
                            if loops >= MAX_ALLOWED_LOOPS {
                                panic!("Too many failed connections");
                            }
                        }

                    },
                    Err(e) => {
                        warn!("Error in event loop, {e}, reconnecting...");
                        if let Ok(websocket) = WebSocketClient::<T>::connect(ssid.clone(), demo).await {
                            (write, read) = websocket.split();
                        } else {
                            loops += 1;
                            if loops >= MAX_ALLOWED_LOOPS {
                                error!("Too many failed connections");
                                break
                            }
                        }

                    }                
                }
            }
        });
        Ok((task, msg_sender))
    }

    async fn listener_loop(data: Data, handler: T, mut previous: MessageInfo, ws: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, sender: &Sender<Message>, msg_sender: &Sender<WebSocketMessage>) -> PocketResult<()> {        
        while let Some(msg) = &ws.next().await {
            println!("Recieved message");
            match msg {
                Ok(msg) => {
                    match handler.process_message(msg, &previous, sender, &data).await {
                        Ok((msg, close)) => {
                            if close {
                                return Ok(())
                            }
                            if let Some(msg) = msg {
                                previous = msg;
                            }
                        },
                        Err(e) => {
                            debug!("Error processing message, {e}");
                        }
                    }
                },
                Err(e) => {
                    warn!("Error recieving websocket message, {e}");
                }
            }
        }
        Ok(())
    }

    async fn sender_loop(ws: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, reciever: &mut Receiver<Message>) -> PocketResult<()> {
        while let Some(msg) = reciever.recv().await {
            match ws.send(msg).await {
                Ok(_) => {
                    debug!("Sent message");
                    println!("Sent message");
                },
                Err(e) => {
                    warn!("Error sending message: {}", e);
                } 
            }
            ws.flush().await?;
        }
        Ok(())
    }

    async fn update_loop(data: Data, reciever: &mut Receiver<WebSocketMessage>, sender: &Sender<Message>) -> PocketResult<()> {
        while let Some(msg) = reciever.recv().await {
            match msg {
                WebSocketMessage::SuccessupdateBalance(balance) => data.update_balance(balance).await,
                WebSocketMessage::UpdateAssets(assets) => data.update_payout_data(assets).await,
                WebSocketMessage::UpdateClosedDeals(deals) => data.update_closed_deals(deals).await,
                WebSocketMessage::UpdateOpenedDeals(deals) => data.update_opened_deals(deals).await,
                WebSocketMessage::UserRequest(request) => {
                    data.add_user_request(request.response_type, request.validator, request.sender).await;
                    sender.send(request.message.into());
                }
                _ => {}
            }
        }
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc, time::Duration};

    use chrono::{format, Utc};
    use serde_json::Value;
    use tokio_tungstenite::{tungstenite::protocol::Message, connect_async_tls_with_config, tungstenite::{error::TlsError, handshake::client::generate_key, http::{Request, Uri}}, Connector};
    use futures_util::{future, pin_mut, SinkExt, StreamExt};
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex, time::sleep};

    use crate::pocketoption::{parser::message::WebSocketMessage, types::info::MessageInfo, utils::basic::get_index, ws::{basic::WebSocketClient, listener::Handler, ssid::Ssid}};

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

        // let client = WebSocketClient { ssid: ssid.clone(), ws: Arc::new(Mutex::new(None)) };
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
            // client.process_message(message.clone(), MessageInfo::None);
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

    #[tokio::test]
    async fn test_websocket_client() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let demo = true;
        let client = WebSocketClient::<Handler>::new(ssid, demo).await?;
        let mut test = 0;
        while test < 1000 {
            test += 1;
            sleep(Duration::from_millis(100)).await;
            println!("{test}");
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