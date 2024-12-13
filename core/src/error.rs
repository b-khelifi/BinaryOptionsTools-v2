use thiserror::Error;

use crate::pocketoption::error::PocketOptionError;

#[derive(Error, Debug)]
pub enum BinaryOptionsToolsError {
    #[error("PocketOptionError, {0}")]
    PocketOptionError(#[from] PocketOptionError),
}
