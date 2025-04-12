use binary_options_tools::{
    error::BinaryOptionsToolsError, pocketoption::error::PocketOptionError,
};
use napi::Error;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum BinaryErrorJs {
    #[error("BinaryOptionsError, {0}")]
    BinaryOptionsError(#[from] BinaryOptionsToolsError),
    #[error("PocketOptionError, {0}")]
    PocketOptionError(#[from] PocketOptionError),
    #[error("Uninitialized, {0}")]
    Uninitialized(String),
    #[error("Error descerializing data, {0}")]
    DeserializingError(#[from] serde_json::Error),
    #[error("UUID parsing error, {0}")]
    UuidParsingError(#[from] uuid::Error),
    #[error("Trade not found, haven't found trade for id '{0}'")]
    TradeNotFound(Uuid),
    #[error("Operation not allowed")]
    NotAllowed(String),
    #[error("Invalid Regex pattern, {0}")]
    InvalidRegexError(#[from] regex::Error),
}

impl From<BinaryErrorJs> for Error {
    fn from(value: BinaryErrorJs) -> Self {
        Error::from_reason(value.to_string())
    }
}
