
use async_channel::Receiver;
use futures_util::stream::{Stream, unfold};
use tracing::debug;
// use pin_project_lite::pin_project;
use crate::pocketoption::{error::PocketResult, parser::message::WebSocketMessage, types::update::DataCandle};


pub struct StreamAsset {
    reciever: Receiver<WebSocketMessage>,
    asset: String
}
    


impl StreamAsset {
    pub fn new(reciever: Receiver<WebSocketMessage>, asset: String) -> Self {
        Self { reciever, asset }
    }

    pub async fn recieve(&self) -> PocketResult<DataCandle> {
        while let Ok(candle) = self.reciever.recv().await {
            debug!(target: "StreamAsset", "Recieved UpdateStream!");
            if let WebSocketMessage::UpdateStream(candle) = candle {
                if let Some(candle) = candle.0.first().take_if(|x| x.active == self.asset) { 
                    return Ok(candle.into());
                }
            }
        }
        
        unreachable!("This should never happen, please contact Rick-29 at https://github.com/Rick-29")
    }

    pub fn to_stream(&self) -> impl Stream<Item = PocketResult<DataCandle>> + '_ {
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