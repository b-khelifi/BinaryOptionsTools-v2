use thiserror::Error;

use tokio_tungstenite::tungstenite::{Error as TungsteniteError, Message};
use tokio::sync::mpsc::error::SendError;

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
    MessageSendingError(#[from] SendError<Message>),
    #[error("Failed to recieve message from separate thread, {0}")]
    OneShotRecieverError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Failed to send message to websocket sender, {0}")]
    ThreadMessageSendingErrorMPCS(String),
    #[error("Error recieving response from server, {0} ,maybe you used invalid data in the request?")]
    WebSocketMessageError(String)
}

pub type BinaryOptionsResult<T> = Result<T, BinaryOptionsToolsError>;

impl<Transfer> From<Transfer> for BinaryOptionsToolsError
where
    Transfer: MessageTransfer,
{
    fn from(value: Transfer) -> Self {
        let error = value.into_error();
        Self::WebsocketMessageSendingError(error.to_string())
    }
}
