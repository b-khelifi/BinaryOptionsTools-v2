use std::time::Duration;

use async_trait::async_trait;
use tokio::time::sleep;
use tracing::{debug, info};

use crate::{
    error::{BinaryOptionsResult, BinaryOptionsToolsError},
    general::{client::SenderMessage, traits::Callback, types::Data},
    pocketoption::{
        error::PocketOptionError, parser::message::WebSocketMessage, types::info::MessageInfo,
        validators::history_validator,
    },
};

use super::{base::ChangeSymbol, data_v2::PocketData, order::SuccessCloseOrder};

#[derive(Clone)]
pub struct PocketCallback;

#[async_trait]
impl Callback for PocketCallback {
    type T = PocketData;
    type Transfer = WebSocketMessage;

    async fn call(
        &self,
        data: Data<Self::T, Self::Transfer>,
        sender: &SenderMessage<Self::Transfer>,
    ) -> BinaryOptionsResult<()> {
        sleep(Duration::from_secs(5)).await;

        for asset in data.stream_assets().await {
            sleep(Duration::from_secs(1)).await;
            let history = ChangeSymbol::new(asset.to_string(), 3600);
            let res = sender
                .send_message(
                    &data,
                    WebSocketMessage::ChangeSymbol(history),
                    MessageInfo::UpdateHistoryNew,
                    history_validator(asset.to_string(), 3600),
                )
                .await?;
            if let WebSocketMessage::UpdateHistoryNew(_) = res {
                debug!("Sent 'ChangeSymbol' for asset: {asset}");
            } else {
                return Err(
                    PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()).into(),
                );
            }
        }
        if let Some(sender) = data.sender(MessageInfo::SuccesscloseOrder).await {
            let deals = data.get_closed_deals().await;
            if !deals.is_empty() {
                info!(target: "PocketCallback", "Sending closed orders data after disconnection");
                let close_order = SuccessCloseOrder { profit: 0.0, deals };
                sender.send(WebSocketMessage::SuccesscloseOrder(close_order)).await.map_err(|e| BinaryOptionsToolsError::ThreadMessageSendingErrorMPCS(e.to_string()))?;
            }
        }

        
        Ok(())
    }
}
