use crate::error::{BinaryOptionsResult, BinaryOptionsToolsError};

use super::traits::MessageTransfer;

pub fn validate<Transfer>(
    validator: impl Fn(&Transfer) -> bool + Send + Sync,
    message: Transfer,
) -> BinaryOptionsResult<Option<Transfer>>
where
    Transfer: MessageTransfer,
{
    if let Some(e) = message.error() {
        Err(BinaryOptionsToolsError::WebSocketMessageError(
            e.to_string(),
        ))
    } else if validator(&message) {
        Ok(Some(message))
    } else {
        Ok(None)
    }
}
