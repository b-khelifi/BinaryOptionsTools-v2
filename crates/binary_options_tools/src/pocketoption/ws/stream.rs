use std::sync::Arc;
use std::time::Duration;

use crate::pocketoption::error::PocketOptionError;
use chrono::{DateTime, Utc};
use tracing::debug;
// use pin_project_lite::pin_project;
use crate::pocketoption::{
    error::PocketResult, parser::message::WebSocketMessage, types::update::DataCandle,
};

use async_channel::Receiver;
use futures_util::stream::unfold;
use futures_util::Stream;

#[derive(Clone)]
pub struct StreamAsset {
    reciever: Receiver<WebSocketMessage>,
    asset: String,
    condition: ConditonnalUpdate,
}

/// This enum tells the StreamAsset when to send new data
#[derive(Clone)]
pub enum ConditonnalUpdate {
    None,           // No condition, once data is recieved, data is sent
    Size(usize),    // Data is only returned when length of candles is equal to size
    Time(Duration), // Only return data when the time between the first and latest candle is equal to the specified duration
}

impl ConditonnalUpdate {
    pub fn check_condition(&self, candles: &[DataCandle]) -> PocketResult<bool> {
        match self {
            Self::None => Ok(true),
            Self::Size(batch) => {
                if candles.len() >= *batch {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::Time(time) => {
                if let Some(first) = candles.first() {
                    let start_time = first.time;
                    let end_time: DateTime<Utc> = match candles.last() {
                        Some(candle) => candle.time,
                        None => return Ok(false),
                    };
                    let duration = (end_time - start_time).to_std().map_err(|_| PocketOptionError::UnreachableError("Well, this is unexpected, somehow the first candle is more recent than the latest one recieved".to_string()))?;
                    if duration >= *time {
                        return Ok(true);
                    }
                    return Ok(false);
                }
                Ok(false)
            }
        }
    }
}

impl StreamAsset {
    pub fn new(reciever: Receiver<WebSocketMessage>, asset: String) -> Self {
        Self {
            reciever,
            asset,
            condition: ConditonnalUpdate::None,
        }
    }

    pub fn new_chuncked(
        reciever: Receiver<WebSocketMessage>,
        asset: String,
        chunk_size: usize,
    ) -> Self {
        Self {
            reciever,
            asset,
            condition: ConditonnalUpdate::Size(chunk_size),
        }
    }

    pub fn new_timed(reciever: Receiver<WebSocketMessage>, asset: String, time: Duration) -> Self {
        Self {
            reciever,
            asset,
            condition: ConditonnalUpdate::Time(time),
        }
    }

    pub async fn recieve(&self) -> PocketResult<DataCandle> {
        let mut candles = vec![];
        while let Ok(candle) = self.reciever.recv().await {
            debug!(target: "StreamAsset", "Recieved UpdateStream!");
            if let WebSocketMessage::UpdateStream(candle) = candle {
                if let Some(candle) = candle.0.first().take_if(|x| x.active == self.asset) {
                    candles.push(candle.into());
                    if self.condition.check_condition(&candles)? {
                        return candles.try_into();
                    }
                }
            }
        }

        unreachable!(
            "This should never happen, please contact Rick-29 at https://github.com/Rick-29"
        )
    }

    // pub async fn _recieve(&self) -> PocketResult<DataCandle> {
    //     while let Ok(candle) = self.reciever.recv().await {
    //         debug!(target: "StreamAsset", "Recieved UpdateStream!");
    //         if let WebSocketMessage::UpdateStream(candle) = candle {
    //             if let Some(candle) = candle.0.first().take_if(|x| x.active == self.asset) {
    //                 return Ok(candle.into());
    //             }
    //         }
    //     }

    //     unreachable!(
    //         "This should never happen, please contact Rick-29 at https://github.com/Rick-29"
    //     )
    // }

    // pub async fn recieve_chunked(&self) -> PocketResult<DataCandle> {
    //     let mut chunk = vec![];
    //     while let Ok(candle) = self.reciever.recv().await {
    //         debug!(target: "StreamAsset", "Recieved UpdateStream!");
    //         if let WebSocketMessage::UpdateStream(candle) = candle {
    //             if let Some(candle) = candle.0.first().take_if(|x| x.active == self.asset) {
    //                 chunk.push(candle.into());
    //                 if chunk.len() >= self.chunk_size {
    //                     return chunk.try_into();
    //                 }
    //             }
    //         }
    //     }

    //     unreachable!(
    //         "This should never happen, please contact Rick-29 at https://github.com/Rick-29"
    //     )
    // }

    pub fn to_stream(&self) -> impl Stream<Item = PocketResult<DataCandle>> + '_ {
        Box::pin(unfold(self, |state| async move {
            let item = state.recieve().await;
            Some((item, state))
        }))
    }

    pub fn to_stream_static(
        self: Arc<Self>,
    ) -> impl Stream<Item = PocketResult<DataCandle>> + 'static {
        Box::pin(unfold(self, |state| async move {
            let item = state.recieve().await;
            Some((item, state))
        }))
    }
}

// impl Stream for StreamAsset {
//     type Item = Candle;

//     fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
//         match self.reciever.recv()

//         }
//     }
// }
