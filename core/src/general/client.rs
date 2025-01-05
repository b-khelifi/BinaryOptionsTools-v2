use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use async_channel::{bounded, Receiver, RecvError, Sender};
use futures_util::future::try_join4;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::contstants::{MAX_CHANNEL_CAPACITY, RECONNECT_CALLBACK};
use crate::error::{BinaryOptionsResult, BinaryOptionsToolsError};
use crate::general::types::MessageType;

use super::traits::{Callback, Connect, Credentials, DataHandler, MessageHandler, MessageTransfer};
use super::types::Data;

const MAX_ALLOWED_LOOPS: u32 = 8;
const SLEEP_INTERVAL: u64 = 2;

#[derive(Clone)]
pub struct WebSocketClient<Transfer, Handler, Connector, Creds, T, C>
where
    Transfer: MessageTransfer,
    Handler: MessageHandler,
    Connector: Connect,
    Creds: Credentials,
    T: DataHandler,
    C: Callback,
{
    inner: Arc<WebSocketInnerClient<Transfer, Handler, Connector, Creds, T, C>>,
}

pub struct WebSocketInnerClient<Transfer, Handler, Connector, Creds, T, C>
where
    Transfer: MessageTransfer,
    Handler: MessageHandler,
    Connector: Connect,
    Creds: Credentials,
    T: DataHandler,
    C: Callback,
{
    pub credentials: Creds,
    pub connector: Connector,
    pub handler: Handler,
    pub data: Data<T, Transfer>,
    pub sender: SenderMessage<Transfer>,
    pub reconnect_callback: Option<C>,
    _event_loop: JoinHandle<BinaryOptionsResult<()>>,
}

impl<Transfer, Handler, Connector, Creds, T, C> Deref
    for WebSocketClient<Transfer, Handler, Connector, Creds, T, C>
where
    Transfer: MessageTransfer,
    Handler: MessageHandler,
    Connector: Connect,
    Creds: Credentials,
    T: DataHandler,
    C: Callback,
{
    type Target = WebSocketInnerClient<Transfer, Handler, Connector, Creds, T, C>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl<Transfer, Handler, Connector, Creds, T, C>
    WebSocketClient<Transfer, Handler, Connector, Creds, T, C>
where
    Transfer: MessageTransfer + 'static,
    Handler: MessageHandler<Transfer = Transfer> + 'static,
    Creds: Credentials + 'static,
    Connector: Connect<Creds = Creds> + 'static,
    T: DataHandler<Transfer = Transfer> + 'static,
    C: Callback<T = T, Transfer = Transfer> + 'static,
{
    pub async fn init(
        credentials: Creds,
        connector: Connector,
        data: Data<T, Transfer>,
        handler: Handler,
        timeout: Duration,
        reconnect_callback: Option<C>,
    ) -> BinaryOptionsResult<Self> {
        let inner = WebSocketInnerClient::init(
            credentials,
            connector,
            data,
            handler,
            timeout,
            reconnect_callback,
        )
        .await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

impl<Transfer, Handler, Connector, Creds, T, C>
    WebSocketInnerClient<Transfer, Handler, Connector, Creds, T, C>
where
    Transfer: MessageTransfer + 'static,
    Handler: MessageHandler<Transfer = Transfer> + 'static,
    Creds: Credentials + 'static,
    Connector: Connect<Creds = Creds> + 'static,
    T: DataHandler<Transfer = Transfer> + 'static,
    C: Callback<T = T, Transfer = Transfer> + 'static,
{
    pub async fn init(
        credentials: Creds,
        connector: Connector,
        data: Data<T, Transfer>,
        handler: Handler,
        timeout: Duration,
        reconnect_callback: Option<C>,
    ) -> BinaryOptionsResult<Self> {
        let _connection = connector.connect(credentials.clone()).await?;
        let (_event_loop, sender) = Self::start_loops(
            handler.clone(),
            credentials.clone(),
            data.clone(),
            connector.clone(),
            reconnect_callback.clone(),
        )
        .await?;
        info!("Started WebSocketClient");
        sleep(timeout).await;
        Ok(Self {
            credentials,
            connector,
            handler,
            data,
            sender,
            reconnect_callback,
            _event_loop,
        })
    }

    async fn start_loops(
        handler: Handler,
        credentials: Creds,
        data: Data<T, Transfer>,
        connector: Connector,
        reconnect_callback: Option<C>,
    ) -> BinaryOptionsResult<(JoinHandle<BinaryOptionsResult<()>>, SenderMessage<Transfer>)> {
        let (mut write, mut read) = connector.connect(credentials.clone()).await?.split();
        let (sender, mut reciever) = bounded(MAX_CHANNEL_CAPACITY);
        let (msg_sender, mut msg_reciever) = bounded(MAX_CHANNEL_CAPACITY);
        let msg_sender = SenderMessage::new(msg_sender).clone();
        let sender_msg = msg_sender.clone();
        let task = tokio::task::spawn(async move {
            let previous = None;
            let mut loops = 0;
            let mut reconnected = false;
            loop {
                let listener_future = WebSocketInnerClient::<
                    Transfer,
                    Handler,
                    Connector,
                    Creds,
                    T,
                    C,
                >::listener_loop(
                    previous.clone(),
                    &data,
                    handler.clone(),
                    &sender,
                    &mut read,
                );
                let sender_future =
                    WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T, C>::sender_loop(
                        &mut write,
                        &mut reciever,
                    );
                let update_loop =
                    WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T, C>::api_loop(
                        &mut msg_reciever,
                        &sender,
                    );
                let callback = WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T, C>::reconnect_callback(reconnect_callback.clone(), data.clone(), sender_msg.clone(), reconnected);

                match try_join4(listener_future, sender_future, update_loop, callback).await {
                    Ok(_) => {
                        if let Ok(websocket) = connector.connect(credentials.clone()).await {
                            (write, read) = websocket.split();
                            info!("Reconnected successfully!");
                            loops = 0;
                            reconnected = true;
                        } else {
                            loops += 1;
                            warn!("Error reconnecting... trying again in {SLEEP_INTERVAL} seconds (try {loops} of {MAX_ALLOWED_LOOPS}");
                            sleep(Duration::from_secs(SLEEP_INTERVAL)).await;
                            if loops >= MAX_ALLOWED_LOOPS {
                                panic!("Too many failed connections");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error in event loop, {e}, reconnecting...");
                        if let Ok(websocket) = connector.connect(credentials.clone()).await {
                            (write, read) = websocket.split();
                            info!("Reconnected successfully!");
                            loops = 0;
                            reconnected = true;
                        } else {
                            loops += 1;
                            warn!("Error reconnecting... trying again in {SLEEP_INTERVAL} seconds (try {loops} of {MAX_ALLOWED_LOOPS}");
                            sleep(Duration::from_secs(SLEEP_INTERVAL)).await;
                            if loops >= MAX_ALLOWED_LOOPS {
                                error!("Too many failed connections");
                                break;
                            }
                        }
                    }
                }
            }
            Ok(())
        });
        Ok((task, msg_sender))
    }

    async fn listener_loop(
        mut previous: Option<<<Handler as MessageHandler>::Transfer as MessageTransfer>::Info>,
        data: &Data<T, Transfer>,
        handler: Handler,
        sender: &Sender<Message>,
        ws: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> BinaryOptionsResult<()> {
        while let Some(msg) = &ws.next().await {
            let msg = msg
                .as_ref()
                .inspect_err(|e| warn!("Error recieving websocket message, {e}"))
                .map_err(|e| {
                    BinaryOptionsToolsError::WebsocketRecievingConnectionError(e.to_string())
                })?;
            match handler.process_message(msg, &previous, sender).await {
                Ok((msg, close)) => {
                    if close {
                        info!("Recieved closing frame");
                        return Err(BinaryOptionsToolsError::WebsocketConnectionClosed(
                            "Recieved closing frame".into(),
                        ));
                    }
                    if let Some(msg) = msg {
                        match msg {
                            MessageType::Info(info) => {
                                debug!("Recieved info: {}", info);
                                previous = Some(info);
                            }
                            MessageType::Transfer(transfer) => {
                                debug!("Recieved data of type: {}", transfer.info());
                                if let Some(senders) = data.update_data(transfer.clone()).await? {
                                    for sender in senders {
                                        sender.send(transfer.clone()).await.map_err(|e| {
                                            BinaryOptionsToolsError::ChannelRequestSendingError(
                                                e.to_string(),
                                            )
                                        })?;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Error processing message, {e}");
                }
            }
        }
        todo!()
    }

    async fn sender_loop(
        ws: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        reciever: &mut Receiver<Message>,
    ) -> BinaryOptionsResult<()> {
        while let Ok(msg) = reciever.recv().await {
            match ws.send(msg).await {
                Ok(_) => debug!("Sent message"),
                Err(e) => {
                    warn!("Error sending messge: {e}");
                    return Err(e.into());
                }
            }
            ws.flush().await?;
        }
        Ok(())
    }

    async fn api_loop(
        reciever: &mut Receiver<Transfer>,
        sender: &Sender<Message>,
    ) -> BinaryOptionsResult<()> {
        while let Ok(msg) = reciever.recv().await {
            sender.send(msg.into()).await?;
        }
        Ok(())
    }

    async fn reconnect_callback(
        reconnect_callback: Option<C>,
        data: Data<T, Transfer>,
        sender: SenderMessage<Transfer>,
        reconnect: bool,
    ) -> BinaryOptionsResult<BinaryOptionsResult<()>> {
        Ok(tokio::spawn(async move {
            sleep(Duration::from_secs(RECONNECT_CALLBACK)).await;
            if reconnect {
                if let Some(callback) = &reconnect_callback {
                    callback.call(data.clone(), &sender).await.inspect_err(
                        |e| error!(target: "EventLoop","Error calling callback, {e}"),
                    )?;
                }
            }
            Ok(())
        })
        .await?)
    }
    pub async fn send_message(
        &self,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync,
    ) -> BinaryOptionsResult<Transfer> {
        self.sender
            .send_message(&self.data, msg, response_type, validator)
            .await
    }
}

pub fn validate<Transfer>(
    validator: impl Fn(&Transfer) -> bool + Send + Sync,
    message: Transfer,
) -> BinaryOptionsResult<Option<Transfer>>
where
    Transfer: MessageTransfer,
{
    if let Some(e) = message.error() {
        Err(BinaryOptionsToolsError::WebSocketMessageError(
            e.to_string(),
        ))
    } else if validator(&message) {
        Ok(Some(message))
    } else {
        Ok(None)
    }
}

#[derive(Clone)]
pub struct SenderMessage<Transfer>
where
    Transfer: MessageTransfer,
{
    sender: Sender<Transfer>,
}

impl<Transfer> SenderMessage<Transfer>
where
    Transfer: MessageTransfer,
{
    fn new(sender: Sender<Transfer>) -> Self {
        Self { sender }
    }

    pub async fn send_message<T: DataHandler<Transfer = Transfer>>(
        &self,
        data: &Data<T, Transfer>,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync,
    ) -> BinaryOptionsResult<Transfer> {
        let reciever = data.add_request(response_type).await;

        self.sender
            .send(msg)
            .await
            .map_err(|e| BinaryOptionsToolsError::ThreadMessageSendingErrorMPCS(e.to_string()))?;

        while let Ok(msg) = reciever.recv().await {
            if let Some(msg) =
                validate(&validator, msg).inspect_err(|e| eprintln!("Failed to place trade {e}"))?
            {
                return Ok(msg);
            }
        }
        Err(BinaryOptionsToolsError::ChannelRequestRecievingError(
            RecvError,
        ))
    }
}
