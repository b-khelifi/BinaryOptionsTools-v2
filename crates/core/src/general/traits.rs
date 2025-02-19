use async_trait::async_trait;
use core::{error, fmt, hash};
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::error::BinaryOptionsResult;

use super::{
    config::Config, send::SenderMessage, types::{Data, MessageType}
};

pub trait Credentials: Clone + Send + Sync + Serialize + DeserializeOwned {}

#[async_trait]
pub trait DataHandler: Clone + Send + Sync {
    type Transfer: MessageTransfer;

    async fn update(&self, message: &Self::Transfer) -> BinaryOptionsResult<()>;
}

#[async_trait]
pub trait WCallback: Send + Sync {
    type T: DataHandler;
    type Transfer: MessageTransfer;

    async fn call(
        &self,
        data: Data<Self::T, Self::Transfer>,
        sender: &SenderMessage,
    ) -> BinaryOptionsResult<()>;
}

pub trait MessageTransfer:
    DeserializeOwned + Clone + Into<Message> + Send + Sync + error::Error + fmt::Debug + fmt::Display
{
    type Error: Into<Self> + Clone + error::Error;
    type TransferError: error::Error;
    type Info: MessageInformation;

    fn info(&self) -> Self::Info;

    fn error(&self) -> Option<Self::Error>;

    fn to_error(&self) -> Self::TransferError;

    fn error_info(&self) -> Option<Vec<Self::Info>>;
}

pub trait MessageInformation:
    Serialize + DeserializeOwned + Clone + Send + Sync + Eq + hash::Hash + fmt::Debug + fmt::Display
{
}

#[async_trait]
/// Every struct that implements MessageHandler will recieve a message and should return
pub trait MessageHandler: Clone + Send + Sync {
    type Transfer: MessageTransfer;

    async fn process_message(
        &self,
        message: &Message,
        previous: &Option<<<Self as MessageHandler>::Transfer as MessageTransfer>::Info>,
        sender: &SenderMessage,
    ) -> BinaryOptionsResult<(Option<MessageType<Self::Transfer>>, bool)>;
}

#[async_trait]
pub trait Connect: Clone + Send + Sync {
    type Creds: Credentials;
    // type Uris: Iterator<Item = String>;

    async fn connect<T: DataHandler, Transfer: MessageTransfer>(
        &self,
        creds: Self::Creds,
        config: &Config<T, Transfer>
    ) -> BinaryOptionsResult<WebSocketStream<MaybeTlsStream<TcpStream>>>;
}
