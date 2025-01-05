use thiserror::Error;

use tokio::sync::mpsc::error::SendError;
use tokio_tungstenite::tungstenite::{http, Error as TungsteniteError, Message};

use crate::{general::traits::MessageTransfer, pocketoption::error::PocketOptionError};

#[derive(Error, Debug)]
pub enum BinaryOptionsToolsError {
    #[error("PocketOptionError, {0}")]
    PocketOptionError(#[from] PocketOptionError),
    #[error("Error sending request, {0}")]
    WebsocketMessageSendingError(String),
    #[error("Failed to recieve data from websocket server: {0}")]
    WebsocketRecievingConnectionError(String),
    #[error("Websocket connection was closed by the server, {0}")]
    WebsocketConnectionClosed(String),
    #[error("Failed to connect to websocket server: {0}")]
    WebsocketConnectionError(#[from] TungsteniteError),
    #[error("Failed to send message to websocket sender, {0}")]
    MessageSendingErrorT(#[from] SendError<Message>),
    #[error("Failed to send message to websocket sender, {0}")]
    MessageSendingError(#[from] async_channel::SendError<Message>),
    #[error("Failed to recieve message from separate thread, {0}")]
    OneShotRecieverError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Failed to recieve message from request channel, {0}")]
    ChannelRequestRecievingError(#[from] async_channel::RecvError),
    #[error("Failed to send message to request channel, {0}")]
    ChannelRequestSendingError(String),
    #[error("Failed to send message to websocket sender, {0}")]
    ThreadMessageSendingErrorMPCS(String),
    #[error("Error recieving response from server, {0}")]
    WebSocketMessageError(String),
    #[error("Failed to parse data: {0}")]
    GeneralParsingError(String),
    #[error("TLS Certificate error, {0}")]
    TLSError(#[from] native_tls::Error),
    #[error("Error making http request: {0}")]
    HTTPError(#[from] http::Error),
    #[error("Unallowed operation, {0}")]
    Unallowed(String),
    #[error("Failed to join thread, {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
}

pub type BinaryOptionsResult<T> = Result<T, BinaryOptionsToolsError>;

impl<Transfer> From<Transfer> for BinaryOptionsToolsError
where
    Transfer: MessageTransfer,
{
    fn from(value: Transfer) -> Self {
        let error = value.to_error();
        Self::WebsocketMessageSendingError(error.to_string())
    }
}
