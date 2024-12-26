use binary_option_tools_core::pocketoption::error::PocketOptionError;
use pyo3::{exceptions::PyValueError, PyErr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinaryErrorPy {
    #[error("PocketOptionError, {0}")]
    PocketOptionError(#[from] PocketOptionError),
    #[error("Uninitialized, {0}")]
    Uninitialized(String),
    #[error("Error descerializing data, {0}")]
    DeserializingError(#[from] serde_json::Error),
    #[error("UUID parsing error, {0}")]
    UuidParsingError(#[from] uuid::Error),
}

impl From<BinaryErrorPy> for PyErr {
    fn from(value: BinaryErrorPy) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

pub type BinaryResultPy<T> = Result<T, BinaryErrorPy>;
