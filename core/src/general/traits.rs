use core::{error, fmt, hash};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::TcpStream, sync::mpsc::Sender};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::error::BinaryOptionsResult;

use super::types::{MessageType, UserRequest};

pub trait Credentials: Clone + Send + Sync {}

#[async_trait]
pub trait DataHandler: Clone + Send + Sync {
    async fn update<Transfer>(&self, message: &Transfer)
    where
        Transfer: MessageTransfer;
}

pub trait MessageTransfer: DeserializeOwned + Clone + Into<Message> + Send + Sync + error::Error + fmt::Debug + fmt::Display {
    type Error: Into<Self> + Clone + error::Error;
    type TransferError: error::Error;
    type None: Into<Self>;

    fn info<Info: MessageInformation>(&self) -> Info;

    fn error(&self) -> Option<Self::Error>;

    fn into_error(&self) -> Self::TransferError;

    fn user_request<Transfer: MessageTransfer, Info: MessageInformation>(&self) -> Option<UserRequest<Transfer, Info>>;

    fn new_user<Transfer: MessageTransfer, Info: MessageInformation>(request: UserRequest<Transfer, Info>) -> Self;
}

pub trait MessageInformation:
    Serialize + DeserializeOwned + Clone + Send + Sync +hash::Hash + Eq + PartialEq + fmt::Debug + fmt::Display
{
    fn none(&self) -> Self;
}


#[async_trait]
/// Every struct that implements MessageHandler will recieve a message and should return
pub trait MessageHandler: Clone + Send + Sync{
    async fn process_message<'i, Transfer, Info>(
        &self,
        message: &Message,
        previous: &Option<Info>,
        sender: &Sender<Message>,
        local_sender: &Sender<Transfer>,
    ) -> BinaryOptionsResult<(Option<MessageType<Transfer, Info>>, bool)>
    where
        Transfer: MessageTransfer,
        Info: MessageInformation;
}

#[async_trait]
pub trait Connect: Clone + Send + Sync {
    async fn connect<Creds: Credentials>(
        &self,
        creds: Creds,
    ) -> BinaryOptionsResult<WebSocketStream<MaybeTlsStream<TcpStream>>>;
}
