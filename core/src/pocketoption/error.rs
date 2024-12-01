use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PocketOptionError {
    #[error("Failed to parse SSID: {0}")]
    SsidParsingError(String),
    #[error("Failed to parse data: {0}")]
    GeneralParsingError(String),
    #[error("Failed to parse recieved data to Message: {0}")]
    WebSocketMessageParsingError(#[from] serde_json::Error),
    #[error("Failed to process recieved Message: {0}")]
    WebSocketMessageProcessingError(#[from] anyhow::Error),
    #[error("Failed to convert bytes to string, {0}")]
    WebSocketMessageByteSerializationError(#[from] FromUtf8Error)
}
