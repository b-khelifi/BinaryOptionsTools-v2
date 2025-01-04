use std::time::Duration;

use async_channel::Sender;
use async_trait::async_trait;
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::{error::{BinaryOptionsResult, BinaryOptionsToolsError}, general::{traits::Callback, types::Data}, pocketoption::parser::message::WebSocketMessage};

use super::{base::ChangeSymbol, data_v2::PocketData};


#[derive(Clone)]
pub struct PocketCallback;

#[async_trait]
impl Callback for PocketCallback {
    type T = PocketData;
    type Transfer = WebSocketMessage;

    async fn call(&self, data: Data<Self::T, Self::Transfer>, sender: &Sender<Self::Transfer>) -> BinaryOptionsResult<()> {
        sleep(Duration::from_secs(5)).await;
        
        for asset in data.stream_assets().await {
            sleep(Duration::from_secs(1)).await;
            warn!("Sent 'ChangeSymbol' for asset: {asset}");
            let history = ChangeSymbol::new(asset.to_string(), 3600);
            sender.send(WebSocketMessage::ChangeSymbol(history)).await.map_err(|e| BinaryOptionsToolsError::ThreadMessageSendingErrorMPCS(e.to_string()))?;
        }
        Ok(())
    }
}