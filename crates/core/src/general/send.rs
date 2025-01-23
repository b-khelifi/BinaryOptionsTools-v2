use std::time::Duration;

use async_channel::{bounded, Receiver, RecvError, Sender};
use tokio_tungstenite::tungstenite::Message;
use tracing::error;

use crate::{
    error::{BinaryOptionsResult, BinaryOptionsToolsError},
    general::validate::validate,
    utils::time::timeout,
};

use super::{
    traits::{DataHandler, MessageTransfer},
    types::Data,
};

#[derive(Clone)]
pub struct SenderMessage {
    sender: Sender<Message>,
    sender_priority: Sender<Message>,
}

impl SenderMessage {
    pub fn new(cap: usize) -> (Self, (Receiver<Message>, Receiver<Message>)) {
        let (s, r) = bounded(cap);
        let (sp, rp) = bounded(cap);

        (
            Self {
                sender: s,
                sender_priority: sp,
            },
            (r, rp),
        )
    }
    // pub fn new(sender: Sender<Transfer>) -> Self {
    //     Self { sender }
    // }
    async fn reciever<Transfer: MessageTransfer, T: DataHandler<Transfer = Transfer>>(
        &self,
        data: &Data<T, Transfer>,
        msg: Transfer,
        response_type: Transfer::Info,
    ) -> BinaryOptionsResult<Receiver<Transfer>> {
        let reciever = data.add_request(response_type).await;

        self.send(msg)
            .await
            .map_err(|e| BinaryOptionsToolsError::ThreadMessageSendingErrorMPCS(e.to_string()))?;
        Ok(reciever)
    }

    pub async fn send<Transfer: MessageTransfer>(&self, msg: Transfer) -> BinaryOptionsResult<()> {
        self.sender
            .send(msg.into())
            .await
            .map_err(|e| BinaryOptionsToolsError::ChannelRequestSendingError(e.to_string()))?;
        Ok(())
    }

    pub async fn priority_send(&self, msg: Message) -> BinaryOptionsResult<()> {
        self.sender_priority
            .send(msg)
            .await
            .map_err(|e| BinaryOptionsToolsError::ChannelRequestSendingError(e.to_string()))?;
        Ok(())
    }

    pub async fn send_message<Transfer: MessageTransfer, T: DataHandler<Transfer = Transfer>>(
        &self,
        data: &Data<T, Transfer>,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync,
    ) -> BinaryOptionsResult<Transfer> {
        let reciever = self.reciever(data, msg, response_type).await?;

        while let Ok(msg) = reciever.recv().await {
            if let Some(msg) =
                validate(&validator, msg).inspect_err(|e| error!("Failed to place trade {e}"))?
            {
                return Ok(msg);
            }
        }
        Err(BinaryOptionsToolsError::ChannelRequestRecievingError(
            RecvError,
        ))
    }

    pub async fn send_message_with_timout<
        Transfer: MessageTransfer,
        T: DataHandler<Transfer = Transfer>,
    >(
        &self,
        time: Duration,
        task: impl ToString,
        data: &Data<T, Transfer>,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync,
    ) -> BinaryOptionsResult<Transfer> {
        let reciever = self.reciever(data, msg, response_type).await?;

        timeout(
            time,
            async {
                while let Ok(msg) = reciever.recv().await {
                    if let Some(msg) = validate(&validator, msg)
                        .inspect_err(|e| eprintln!("Failed to place trade {e}"))?
                    {
                        return Ok(msg);
                    }
                }
                Err(BinaryOptionsToolsError::ChannelRequestRecievingError(
                    RecvError,
                ))
            },
            task.to_string(),
        )
        .await
    }

    pub async fn send_message_with_timeout_and_retry<
        Transfer: MessageTransfer,
        T: DataHandler<Transfer = Transfer>,
    >(
        &self,
        time: Duration,
        task: impl ToString,
        data: &Data<T, Transfer>,
        msg: Transfer,
        response_type: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync,
    ) -> BinaryOptionsResult<Transfer> {
        let reciever = self
            .reciever(data, msg.clone(), response_type.clone())
            .await?;

        let call1 = timeout(
            time,
            async {
                while let Ok(msg) = reciever.recv().await {
                    if let Some(msg) = validate(&validator, msg)
                        .inspect_err(|e| eprintln!("Failed to place trade {e}"))?
                    {
                        return Ok(msg);
                    }
                }
                Err(BinaryOptionsToolsError::ChannelRequestRecievingError(
                    RecvError,
                ))
            },
            task.to_string(),
        )
        .await;
        match call1 {
            Ok(res) => Ok(res),
            Err(_) => {
                println!("Failded once trying again");
                let reciever = self.reciever(data, msg, response_type).await?;
                timeout(
                    time,
                    async {
                        while let Ok(msg) = reciever.recv().await {
                            if let Some(msg) = validate(&validator, msg)
                                .inspect_err(|e| eprintln!("Failed to place trade {e}"))?
                            {
                                return Ok(msg);
                            }
                        }
                        Err(BinaryOptionsToolsError::ChannelRequestRecievingError(
                            RecvError,
                        ))
                    },
                    task.to_string(),
                )
                .await
            }
        }
    }
}
