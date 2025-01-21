use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, RecvError};
use futures_util::future::try_join3;
use futures_util::stream::{select_all, SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::constants::MAX_CHANNEL_CAPACITY;
use crate::error::{BinaryOptionsResult, BinaryOptionsToolsError};
use crate::general::stream::RecieverStream;
use crate::general::types::MessageType;

use super::send::SenderMessage;
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
    pub sender: SenderMessage,
    pub reconnect_callback: Option<C>,
    pub reconnect_time: u64,
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
        reconnect_time: Option<u64>,
    ) -> BinaryOptionsResult<Self> {
        let inner = WebSocketInnerClient::init(
            credentials,
            connector,
            data,
            handler,
            timeout,
            reconnect_callback,
            reconnect_time.unwrap_or_default()
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
        reconnect_time: u64
    ) -> BinaryOptionsResult<Self> {
        let _connection = connector.connect(credentials.clone()).await?;
        let (_event_loop, sender) = Self::start_loops(
            handler.clone(),
            credentials.clone(),
            data.clone(),
            connector.clone(),
            reconnect_callback.clone(),
            reconnect_time,
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
            reconnect_time,
            _event_loop,
        })
    }

    async fn start_loops(
        handler: Handler,
        credentials: Creds,
        data: Data<T, Transfer>,
        connector: Connector,
        reconnect_callback: Option<C>,
        time: u64,
    ) -> BinaryOptionsResult<(JoinHandle<BinaryOptionsResult<()>>, SenderMessage)> {
        let (mut write, mut read) = connector.connect(credentials.clone()).await?.split();
        let (sender, (reciever, reciever_priority)) = SenderMessage::new(MAX_CHANNEL_CAPACITY);
        let loop_sender = sender.clone();
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
                    &loop_sender,
                    &mut read,
                );
                let sender_future =
                    WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T, C>::sender_loop(
                        &mut write,
                        &reciever,
                        &reciever_priority,
                        time
                    );

                // let update_loop =
                //     WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T, C>::api_loop(
                //         &mut msg_reciever,
                //         &sender,
                //     );
                let callback = WebSocketInnerClient::<Transfer, Handler, Connector, Creds, T, C>::reconnect_callback(reconnect_callback.clone(), data.clone(), loop_sender.clone(), reconnected, time);

                match try_join3(listener_future, sender_future, callback).await {
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
                        // println!("Reconnecting...");
                        if let Ok(websocket) = connector.connect(credentials.clone()).await {
                            (write, read) = websocket.split();
                            info!("Reconnected successfully!");
                            // println!("Reconnected successfully!");
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
        Ok((task, sender))
    }

    /// Recieves all the messages from the websocket connection and handles it 
    async fn listener_loop(
        mut previous: Option<<<Handler as MessageHandler>::Transfer as MessageTransfer>::Info>,
        data: &Data<T, Transfer>,
        handler: Handler,
        sender: &SenderMessage,
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

    /// Recieves all the messages and sends them to the websocket
    async fn sender_loop(
        ws: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        reciever: &Receiver<Message>,
        reciever_priority: &Receiver<Message>,
        time: u64
    ) -> BinaryOptionsResult<()> {
        async fn priority_mesages(ws: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, reciever_priority: &Receiver<Message>) -> BinaryOptionsResult<()> {
            while let Ok(msg) = reciever_priority.recv().await {
                ws.send(msg).await.inspect_err(|e| warn!("Error sending message to websocket, {e}"))?;
                ws.flush().await?;
                debug!("Sent message to websocket!");
            }
            Err(BinaryOptionsToolsError::ChannelRequestRecievingError(RecvError))
        }

        tokio::select! {
            res = priority_mesages(ws, reciever_priority) => res?,
            _ = sleep(Duration::from_secs(time)) => {}
        }
        let stream1 = RecieverStream::new(reciever.to_owned());
        let stream2 = RecieverStream::new(reciever_priority.to_owned());
        let mut fused_streams = select_all([stream1.to_stream(), stream2.to_stream()]);

        while let Some(Ok(msg)) = fused_streams.next().await {
            ws.send(msg).await.inspect_err(|e| warn!("Error sending message to websocket, {e}"))?;
            ws.flush().await?;
            debug!("Sent message to websocket!");
        }
        Err(BinaryOptionsToolsError::ChannelRequestRecievingError(RecvError))
    }

    // async fn api_loop(
    //     reciever: &mut Receiver<Transfer>,
    //     sender: &Sender<Message>,
    // ) -> BinaryOptionsResult<()> {
    //     while let Ok(msg) = reciever.recv().await {
    //         sender.send(msg.into()).await?;
    //     }
    //     Ok(())
    // }

    async fn reconnect_callback(
        reconnect_callback: Option<C>,
        data: Data<T, Transfer>,
        sender: SenderMessage,
        reconnect: bool,
        reconnect_time: u64
    ) -> BinaryOptionsResult<BinaryOptionsResult<()>> {
        Ok(tokio::spawn(async move {
            sleep(Duration::from_secs(reconnect_time)).await;
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

    pub async fn send_message_with_timout(
        &self,
        timeout: Duration,
        task: impl ToString,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync,
    ) -> BinaryOptionsResult<Transfer> {
        self.sender
            .send_message_with_timout(timeout, task, &self.data, msg, response_type, validator)
            .await
    }

    pub async fn send_message_with_timeout_and_retry(
        &self,
        timeout: Duration,
        task: impl ToString,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync,
    ) -> BinaryOptionsResult<Transfer> {
        self.sender
            .send_message_with_timeout_and_retry(
                timeout,
                task,
                &self.data,
                msg,
                response_type,
                validator,
            )
            .await
    }
}




// impl<Transfer, Handler, Connector, Creds, T, C> Drop
//     for WebSocketClient<Transfer, Handler, Connector, Creds, T, C>
// where
//     Transfer: MessageTransfer,
//     Handler: MessageHandler,
//     Connector: Connect,
//     Creds: Credentials,
//     T: DataHandler,
//     C: Callback,
// {
//     fn drop(&mut self) {
//         self._event_loop.abort();
//         info!(target: "Drop", "Dropping WebSocketClient instance");
//     }
// }

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_channel::{bounded, Receiver, Sender};
    use futures_util::{future::try_join, stream::{select_all, unfold}, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use tokio::time::sleep;
    use tracing::info;

    use crate::utils::tracing::start_tracing;

    struct RecieverStream<T> {
        inner: Receiver<T>
    }

    impl<T> RecieverStream<T> {
        fn new(inner: Receiver<T>) -> Self {
            Self { inner }
        }

        async fn receive(&self) -> anyhow::Result<T> {
            Ok(self.inner.recv().await?)
        }

        fn to_stream(&self) -> impl Stream<Item = anyhow::Result<T>> + '_ {
            Box::pin(unfold(self, |state| async move {
                let item = state.receive().await;
                Some((item, state))
            }))        
        }
        
    }

    async fn recieve_dif(reciever: Receiver<String>, receiver_priority: Receiver<String>) -> anyhow::Result<()> {
        async fn receiv(r: &Receiver<String>) -> anyhow::Result<()> {
            while let Ok(t) = r.recv().await {
                info!(target: "High priority", "Recieved: {}", t);
            }
            Ok(())
        }
        tokio::select! {
            err = receiv(&receiver_priority) => err?,
            _ = tokio::time::sleep(Duration::from_secs(5)) => {}
        }
        let receiver = RecieverStream::new(reciever);
        let receiver_priority = RecieverStream::new(receiver_priority);
        let mut fused = select_all([receiver.to_stream(), receiver_priority.to_stream()]);
        while let Some(value) = fused.next().await {
            info!(target: "Fused", "Recieved: {}", value?);
        }
    
        Ok(())
    }


    async fn recieve_dif_err(reciever: Receiver<String>, receiver_priority: Receiver<String>) -> anyhow::Result<()> {
        async fn receiv(r: &Receiver<String>) -> anyhow::Result<()> {
            let mut loops = 0;
            while let Ok(t) = r.recv().await {
                if loops == 2 {
                    return Err(anyhow::Error::msg("error receiving message"))
                }
                loops += 1;
                info!(target: "High priority", "Recieved: {}", t);
            }
            Ok(())
        }
        tokio::select! {
            err = receiv(&receiver_priority) => err?,
            _ = tokio::time::sleep(Duration::from_secs(5)) => {}
        }
        let receiver = RecieverStream::new(reciever);
        let receiver_priority = RecieverStream::new(receiver_priority);
        let mut fused = select_all([receiver.to_stream(), receiver_priority.to_stream()]);
        while let Some(value) = fused.next().await {
            info!(target: "Fused", "Recieved: {}", value?);
        }
    
        Ok(())
    }

    async fn sender_dif(sender: Sender<String>, sender_priority: Sender<String>) -> anyhow::Result<()> {
        loop {
            let s1: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
            let s2: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
            sender.send(s1).await?;
            sender_priority.send(s2).await?;
            sleep(Duration::from_secs(1)).await;
        }
    }


    #[tokio::test]
    async fn test_multi_priority_reciever_ok() -> anyhow::Result<()> {
        start_tracing(true)?;
        let (s, r) = bounded(8);
        let (sp, rp) = bounded(8);
        try_join(sender_dif(s, sp), recieve_dif(r, rp)).await?;
        Ok(())
    }

    #[tokio::test]
    #[should_panic(expected = "error receiving message")]
    async fn test_multi_priority_reciever_err() {
        start_tracing(true).unwrap();
        let (s, r) = bounded(8);
        let (sp, rp) = bounded(8);
        try_join(sender_dif(s, sp), recieve_dif_err(r, rp)).await.unwrap();
    }

}
