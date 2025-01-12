use std::time::Duration;

use crate::error::{BinaryOptionsResult, BinaryOptionsToolsError};
use core::future::Future;

pub async fn timeout<F, T>(duration: Duration, future: F, task: String) -> BinaryOptionsResult<T>
where
    F: Future<Output = BinaryOptionsResult<T>>,
{
    let res = tokio::select! {
        _ = tokio::time::sleep(duration) => Err(BinaryOptionsToolsError::TimeoutError { task, duration }),
        result = future => match result {
            Ok(value) => Ok(value),
            Err(err) => Err(err),
        },
    };
    res
}
