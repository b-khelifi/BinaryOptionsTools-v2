use thiserror::Error;

#[derive(Error, Debug)]
pub enum PocketOptionError {
    #[error("Failed to parse SSID: {0}")]
    SsidParsingError(String),
    #[error("Failed to parse data: {0}")]
    GeneralParsingError(String),
}
