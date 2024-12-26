use std::time::Duration;

use futures_util::{
    future::try_join3,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::TcpStream,
    task::JoinHandle,
    time::sleep,
};
use async_channel::{Receiver, Sender, bounded};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::pocketoption::{
    error::{PocketOptionError, PocketResult},
    parser::message::WebSocketMessage,
    types::{data::Data, info::MessageInfo},
    utils::connect::try_connect,
};

use super::{
    listener::{EventListener, Handler},
    ssid::Ssid,
};

const MAX_ALLOWED_LOOPS: u32 = 8;
const SLEEP_INTERVAL: u32 = 2;

pub struct WebSocketClient<T: EventListener> {
    pub ssid: Ssid,
    pub handler: T,
    // pub balance: UpdateBalance
    pub data: Data,
    pub sender: Sender<WebSocketMessage>,
    _event_loop: JoinHandle<()>,
}

impl<T: EventListener> WebSocketClient<T> {
    pub async fn new(ssid: impl ToString) -> PocketResult<WebSocketClient<Handler>> {
        let handler = Handler::new(Ssid::parse(ssid.to_string().clone())?);
        WebSocketClient::init(ssid, handler).await
    }

    pub async fn init(ssid: impl ToString, handler: T) -> PocketResult<Self> {
        let ssid = Ssid::parse(ssid)?;
        let _connection = Self::connect(ssid.clone()).await?;
        let data = Data::default();
        let (_event_loop, sender) =
            Self::start_loops(handler.clone(), ssid.clone(), data.clone()).await?;
        println!("Initialized");
        sleep(Duration::from_millis(500)).await;
        Ok(Self {
            ssid,
            handler,
            data,
            sender,
            _event_loop,
        })
    }

    pub async fn connect(
        ssid: Ssid,
    ) -> PocketResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let urls = ssid.servers().await?;
        let mut error = None;
        for url in urls.clone() {
            match try_connect(ssid.clone(), url).await {
                Ok(connect) => return Ok(connect),
                Err(e) => {
                    warn!("Failed to connect to server, {e}");
                    error = Some(e);
                }
            }
        }
        if let Some(error) = error {
            Err(error)
        } else {
            Err(PocketOptionError::WebsocketMultipleAttemptsConnectionError(
                format!("Couldn't connect to server after {} attempts.", urls.len()),
            ))
        }
    }

    async fn start_loops(
        handler: T,
        ssid: Ssid,
        data: Data,
    ) -> PocketResult<(JoinHandle<()>, Sender<WebSocketMessage>)> {
        let (mut write, mut read) = WebSocketClient::<T>::connect(ssid.clone())
            .await?
            .split();
        let (sender, mut reciever) = bounded(128);
        let (msg_sender, mut msg_reciever) = bounded(128);
        let sender_msg = msg_sender.clone();

        let task = tokio::task::spawn(async move {
            let previous = MessageInfo::None;
            let mut loops = 0;
            loop {
                let listener_future = WebSocketClient::<T>::listener_loop(
                    data.clone(),
                    handler.clone(),
                    previous.clone(),
                    &mut read,
                    &sender,
                    &sender_msg,
                );
                let sender_future = WebSocketClient::<T>::sender_loop(&mut write, &mut reciever);
                let update_loop =
                    WebSocketClient::<T>::update_loop(data.clone(), &mut msg_reciever, &sender);
                match try_join3(listener_future, sender_future, update_loop).await {
                    Ok(_) => {
                        if let Ok(websocket) =
                            WebSocketClient::<T>::connect(ssid.clone()).await
                        {
                            (write, read) = websocket.split();
                            info!("Reconnected successfully!");
                            loops = 0;
                        } else {
                            loops += 1;
                            warn!("Error reconnecting... trying again in {SLEEP_INTERVAL} seconds (try {loops} of {MAX_ALLOWED_LOOPS}");
                            if loops >= MAX_ALLOWED_LOOPS {
                                panic!("Too many failed connections");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error in event loop, {e}, reconnecting...");
                        if let Ok(websocket) =
                            WebSocketClient::<T>::connect(ssid.clone()).await
                        {
                            (write, read) = websocket.split();
                            info!("Reconnected successfully!");
                            loops = 0;
                        } else {
                            loops += 1;
                            warn!("Error reconnecting... trying again in {SLEEP_INTERVAL} seconds (try {loops} of {MAX_ALLOWED_LOOPS}");
                            if loops >= MAX_ALLOWED_LOOPS {
                                error!("Too many failed connections");
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok((task, msg_sender))
    }

    async fn listener_loop(
        data: Data,
        handler: T,
        mut previous: MessageInfo,
        ws: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        sender: &Sender<Message>,
        msg_sender: &Sender<WebSocketMessage>,
    ) -> PocketResult<()> {
        while let Some(msg) = &ws.next().await {
            let msg = msg
                .as_ref()
                .inspect_err(|e| {
                    warn!("Error recieving websocket message, {e}");
                })
                .map_err(|e| PocketOptionError::WebsocketRecievingConnectionError(e.to_string()))?;
            match handler
                .process_message(msg, &previous, sender, msg_sender, &data)
                .await
            {
                Ok((msg, close)) => {
                    if close {
                        info!("Recieved closing frame.");
                        return Err(PocketOptionError::WebsocketConnectionClosed(
                            "Recieved closing frame".into(),
                        ));
                    }
                    if let Some(msg) = msg {
                        previous = msg;
                    }
                }
                Err(e) => {
                    debug!("Error processing message, {e}");
                }
            }
        }
        Ok(())
    }

    async fn sender_loop(
        ws: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        reciever: &mut Receiver<Message>,
    ) -> PocketResult<()> {
        while let Ok(msg) = reciever.recv().await {
            match ws.send(msg).await {
                Ok(_) => {
                    debug!("Sent message");
                }
                Err(e) => {
                    warn!("Error sending message: {}", e);
                    return Err(e.into());
                }
            }
            ws.flush().await?;
        }
        Ok(())
    }

    async fn update_loop(
        data: Data,
        reciever: &mut Receiver<WebSocketMessage>,
        sender: &Sender<Message>,
    ) -> PocketResult<()> {
        while let Ok(msg) = reciever.recv().await {
            match msg {
                WebSocketMessage::SuccessupdateBalance(balance) => {
                    data.update_balance(balance).await
                }
                WebSocketMessage::UpdateAssets(assets) => data.update_payout_data(assets).await,
                WebSocketMessage::UpdateClosedDeals(deals) => {
                    data.update_closed_deals(deals.0).await
                }
                WebSocketMessage::UpdateOpenedDeals(deals) => {
                    data.update_opened_deals(deals.0).await
                }
                WebSocketMessage::SuccesscloseOrder(order) => {
                    data.update_closed_deals(order.deals).await
                }
                WebSocketMessage::SuccessopenOrder(order) => {
                    data.update_opened_deals(vec![order]).await
                },
                WebSocketMessage::UpdateStream(stream) => {
                    debug!("Recieved update stream");
                    if let Err(e) = data.send_stream(stream).await {
                        warn!("Error sending message to StreamAssets, {e}");
                    } else {
                        debug!("Send update stream successfully");
                    }
                }
                WebSocketMessage::UserRequest(request) => {
                    data.add_user_request(request.info, request.validator, request.sender)
                        .await;
                    if request.message.info() == WebSocketMessage::None.info() {
                        continue;
                    }
                    if let Err(e) = sender.send(request.message.into()).await {
                        warn!("Error sending message: {}", PocketOptionError::from(e));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use chrono::Utc;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::Value;
    use tokio_tungstenite::{
        connect_async_tls_with_config,
        tungstenite::protocol::Message,
        tungstenite::{handshake::client::generate_key, http::Request},
        Connector,
    };

    use crate::pocketoption::{
        parser::message::WebSocketMessage, types::info::MessageInfo, utils::basic::get_index,
        ws::ssid::Ssid,
    };

    use crate::pocketoption::parser::basic::LoadHistoryPeriod;

    fn get_candles() -> Result<String, Box<dyn Error>> {
        let time = Utc::now().timestamp();
        let period = 60;
        let offset = 900;
        let history_period = LoadHistoryPeriod {
            asset: "AUDNZD_otc".into(),
            period,
            time,
            index: get_index()?,
            offset,
        };
        Ok(serde_json::to_string(&history_period)?)
    }

    #[tokio::test]
    async fn test_connect() -> Result<(), Box<dyn Error>> {
        let tls_connector = native_tls::TlsConnector::builder().build().unwrap();

        let connector = Connector::NativeTls(tls_connector);
        let ssid: Ssid = Ssid::parse(
            r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#,
        )?;

        // let client = WebSocketClient { ssid: ssid.clone(), ws: Arc::new(Mutex::new(None)) };
        let url =
            url::Url::parse("wss://demo-api-eu.po.market/socket.io/?EIO=4&transport=websocket")?;
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
        let msg = format!("[loadHistoryPeriod, {}]", get_candles()?);
        dbg!(&msg);
        // write.send(Message::text(msg)).await?;
        // write.flush().await?;
        println!("sent");
        let mut lop = 0;
        let assets = ["EURTRY_otc", "EURUSD_otc", "#FB_otc", "CADJPY_otc", "100GBP_otc"];
        let mut inner = 0;
        while let Some(msg) = read.next().await {
            lop += 1;
            if lop % 5 == 0 {
                inner += 1;
                write
                    .send(Message::text(
                        format!("42[\"changeSymbol\",{{\"asset\":\"{}\",\"period\":3600}}]", assets[inner % 5])
                    ))
                    .await
                    .unwrap();
                write.flush().await.unwrap();
                println!("Send subscribeSymbol");
            }
            println!("receiving...");
            let message = msg.unwrap();
            // client.process_message(message.clone(), MessageInfo::None);
            let _msg = match message {
                Message::Binary(bin) | Message::Ping(bin) | Message::Pong(bin) => {
                    let msg = String::from_utf8(bin.to_vec()).unwrap();
                    let _parsed = WebSocketMessage::parse(&msg);
                    // dbg!(parsed);
                    if msg.len() > 64 {
                        dbg!(format!("Bin: {}", &msg[..64]))
                    } else {
                        dbg!(format!("Bin: {}", &msg))
                    }
                }
                Message::Text(text) => {
                    let base = text.clone();
                    match base {
                        _ if base.starts_with('0') && base.contains("sid") => {
                            write.send(Message::text("40")).await.unwrap();
                            write.flush().await.unwrap();
                        }
                        _ if base.starts_with("40") && base.contains("sid") => {
                            write.send(Message::text(ssid.to_string())).await.unwrap();
                            write.flush().await.unwrap();
                        }
                        _ if base == "2" => {
                            write.send(Message::text("3")).await.unwrap();
                            write.flush().await.unwrap();
                        }
                        _ if base.starts_with("451-") => {
                            let msg = base.strip_prefix("451-").unwrap();
                            let (info, _): (MessageInfo, Value) = serde_json::from_str(msg)?;
                            println!("Recieved message: {}", info)
                        }
                        _ => {}
                    }

                    text.to_string()
                }
                Message::Close(_) => String::from("Closed"),
                Message::Frame(_) => unimplemented!(),
            };
        }

        Ok(())
    }

    #[test]
    fn test_bytes() -> Result<(), Box<dyn Error>> {
        let bits = vec![
            77, 105, 115, 115, 105, 110, 103, 32, 111, 114, 32, 105, 110, 118, 97, 108, 105, 100,
            32, 83, 101, 99, 45, 87, 101, 98, 83, 111, 99, 107, 101, 116, 45, 75, 101, 121, 32,
            104, 101, 97, 100, 101, 114,
        ];
        let string = String::from_utf8(bits)?;
        dbg!(string);
        Ok(())
    }
}
