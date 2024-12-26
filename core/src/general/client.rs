use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use futures_util::future::try_join3;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use async_channel::{bounded, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::error::{BinaryOptionsResult, BinaryOptionsToolsError};
use crate::general::types::{MessageType, UserRequest};

use super::traits::{Connect, Credentials, DataHandler, MessageHandler, MessageTransfer};
use super::types::Data;

const MAX_ALLOWED_LOOPS: u32 = 8;
const SLEEP_INTERVAL: u32 = 2;

#[derive(Clone)]
pub struct WebSocketClient<Transfer, Handler, Connector, Creds, T>
where
    Transfer: MessageTransfer,
    Handler: MessageHandler,
    Connector: Connect,
    Creds: Credentials,
    T: DataHandler,
{
    inner: Arc<WebSocketInnerClient<Transfer, Handler, Connector, Creds, T>>,
}

pub struct WebSocketInnerClient<Transfer, Handler, Connector, Creds, T>
where
    Transfer: MessageTransfer,
    Handler: MessageHandler,
    Connector: Connect,
    Creds: Credentials,
    T: DataHandler,
{
    pub credentials: Creds,
    pub connector: Connector,
    pub handler: Handler,
    pub data: Data<T, Transfer>,
    pub sender: Sender<Transfer>,
    _event_loop: JoinHandle<()>,
}

impl<Transfer, Handler, Connector, Creds, T> Deref
    for WebSocketClient<Transfer, Handler, Connector, Creds, T>
where
    Transfer: MessageTransfer,
    Handler: MessageHandler,
    Connector: Connect,
    Creds: Credentials,
    T: DataHandler,
{
    type Target = WebSocketInnerClient<Transfer, Handler, Connector, Creds, T>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl<Transfer, Handler, Connector, Creds, T> WebSocketClient<Transfer, Handler, Connector, Creds, T>
where
    Transfer: MessageTransfer + 'static,
    Handler: MessageHandler<Transfer = Transfer> + 'static,
    Creds: Credentials + 'static,
    Connector: Connect<Creds = Creds> + 'static,
    T: DataHandler<Transfer = Transfer> + 'static,
{
    pub async fn init(
        credentials: Creds,
        connector: Connector,
        data: Data<T, Transfer>,
        handler: Handler,
        timeout: Duration,
    ) -> BinaryOptionsResult<Self> {
        let inner =
            WebSocketInnerClient::init(credentials, connector, data, handler, timeout).await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

impl<Transfer, Handler, Connector, Creds, T>
    WebSocketInnerClient<Transfer, Handler, Connector, Creds, T>
where
    Transfer: MessageTransfer + 'static,
    Handler: MessageHandler<Transfer = Transfer> + 'static,
    Creds: Credentials + 'static,
    Connector: Connect<Creds = Creds> + 'static,
    T: DataHandler<Transfer = Transfer> + 'static,
{
    pub async fn init(
        credentials: Creds,
        connector: Connector,
        data: Data<T, Transfer>,
        handler: Handler,
        timeout: Duration,
    ) -> BinaryOptionsResult<Self> {
        let _connection = connector.connect(credentials.clone()).await?;
        let (_event_loop, sender) = Self::start_loops(
            handler.clone(),
            credentials.clone(),
            data.clone(),
            connector.clone(),
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
            _event_loop,
        })
    }

    async fn start_loops(
        handler: Handler,
        credentials: Creds,
        data: Data<T, Transfer>,
        connector: Connector,
    ) -> BinaryOptionsResult<(JoinHandle<()>, Sender<Transfer>)> {
        let (mut write, mut read) = connector.connect(credentials.clone()).await?.split();
        let (sender, mut reciever) = bounded(128);
        let (msg_sender, mut msg_reciever) = bounded(128);
        let task = tokio::task::spawn(async move {
            let previous = None;
            let mut loops = 0;
            loop {
                let listener_future =
                    WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T>::listener_loop(
                        previous.clone(),
                        &data,
                        handler.clone(),
                        &sender,
                        &mut read,
                    );
                let sender_future =
                    WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T>::sender_loop(
                        &data,
                        &mut write,
                        &mut reciever,
                    );
                let update_loop =
                    WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T>::api_loop(
                        &data,
                        &mut msg_reciever,
                        &sender,
                    );
                match try_join3(listener_future, sender_future, update_loop).await {
                    Ok(_) => {
                        if let Ok(websocket) = connector.connect(credentials.clone()).await {
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
                        if let Ok(websocket) = connector.connect(credentials.clone()).await {
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
                                data.update_data(transfer, sender).await?;
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
        data: &Data<T, Transfer>,
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
            data.list_pending_requests().await;
        }
        Ok(())
    }

    async fn api_loop(
        data: &Data<T, Transfer>,
        reciever: &mut Receiver<Transfer>,
        sender: &Sender<Message>,
    ) -> BinaryOptionsResult<()> {
        while let Ok(msg) = reciever.recv().await {
            if let Some(request) = msg.user_request() {
                data.add_user_request(request.info, request.validator, request.sender)
                    .await;
                let message = *request.message;
                if let Err(e) = sender.send(message.into()).await {
                    warn!(
                        "Error sending message: {}",
                        BinaryOptionsToolsError::from(e)
                    );
                }
                warn!("Add new data");
                data.list_pending_requests().await;
            }
            // data.update_data(msg, sender).await?;
        }
        Ok(())
    }

    pub async fn send_message(
        &self,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync + 'static,
    ) -> BinaryOptionsResult<Transfer> {
        let (request, reciever) = UserRequest::new(msg, response_type, validator);
        debug!(
            "Sending request from user, expecting response: {}",
            request.info
        );
        self.sender
            .send(Transfer::new_user(request))
            .await
            .map_err(|e| BinaryOptionsToolsError::ThreadMessageSendingErrorMPCS(e.to_string()))?;
        let resp = reciever.await?;
        if let Some(e) = resp.error() {
            Err(BinaryOptionsToolsError::WebSocketMessageError(
                e.to_string(),
            ))
        } else {
            Ok(resp)
        }
    }
}
