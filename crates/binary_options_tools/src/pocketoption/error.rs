use std::error::Error;
use std::string::FromUtf8Error;

use super::types::order::PocketMessageFail;
use super::{parser::message::WebSocketMessage, types::info::MessageInfo};
use binary_options_tools_core::error::BinaryOptionsToolsError;
use thiserror::Error;
// use tokio_tungstenite::tungstenite::Error as TungsteniteError;
// use tokio_tungstenite::tungstenite::{http, Message};

#[derive(Error, Debug)]
pub enum PocketOptionError {
    #[error("BinaryOptionsToolsError, {0}")]
    BinaryOptionsToolsError(#[from] BinaryOptionsToolsError),
    #[error("Failed to parse SSID: {0}")]
    SsidParsingError(String),
    #[error("Failed to parse data: {0}")]
    GeneralParsingError(String),
    // #[error("Error making http request: {0}")]
    // HTTPError(#[from] http::Error),
    #[error("TLS Certificate error, {0}")]
    TLSError(#[from] native_tls::Error),
    // #[error("Failed to connect to websocket server: {0}")]
    // WebsocketConnectionError(#[from] TungsteniteError),
    #[error("Failed to connect to websocket server: {0}")]
    WebsocketRecievingConnectionError(String),
    #[error("Websocket connection was closed by the server, {0}")]
    WebsocketConnectionClosed(String),
    #[error("Failed to connect to websocket server, {0}")]
    WebsocketConnectionAttempFailed(String),
    #[error("Failed to connect to websocket server after multiple attempts, {0}")]
    WebsocketMultipleAttemptsConnectionError(String),
    #[error("Failed to parse recieved data to Message: {0}")]
    WebSocketMessageParsingError(#[from] serde_json::Error),
    #[error("Failed to process recieved Message: {0}")]
    WebSocketMessageProcessingError(#[from] anyhow::Error),
    #[error("Failed to convert bytes to string, {0}")]
    WebSocketMessageByteSerializationError(#[from] FromUtf8Error),
    // #[error("Failed to send message to websocket sender, {0}")]
    // MessageSendingError(#[from] async_channel::SendError<Message>),
    #[error("Failed to send message to websocket sender, {0}")]
    ThreadMessageSendingErrorMPCS(#[from] async_channel::SendError<WebSocketMessage>),
    #[error("Failed to recieve message from separate thread, {0}")]
    OneShotRecieverError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Failed to send message to websocket sender, {0}")]
    ThreadMessageSendingError(#[from] WebSocketMessage),
    #[error("Failed to make request, {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Unexpected error, recieved incorrect WebSocketMessage type, recieved {0}")]
    UnexpectedIncorrectWebSocketMessage(#[from] MessageInfo),
    #[error("If you are having this error please contact the developpers, {0}")]
    UnreachableError(String),
    #[error("Unallowed operation, {0}")]
    Unallowed(String),
    #[error("Error sending request, {0}")]
    WebsocketMessageSendingError(#[from] PocketMessageFail),
    #[error("Expected the data to be non-empty for type '{0}'")]
    EmptyArrayError(String),
    #[error("General compiling error: {0}")]
    CompilingError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type PocketResult<T> = Result<T, PocketOptionError>;

impl Error for WebSocketMessage {}
impl Error for MessageInfo {}
impl Error for PocketMessageFail {}

impl From<PocketOptionError> for BinaryOptionsToolsError {
    fn from(value: PocketOptionError) -> Self {
        BinaryOptionsToolsError::BinaryOptionsTradingError {
            platform: "Pocket Option".to_string(),
            error: value.to_string(),
        }
    }
}
