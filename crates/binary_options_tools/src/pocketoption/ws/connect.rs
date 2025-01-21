use async_trait::async_trait;
use tokio::net::TcpStream;
use tracing::warn;

use crate::pocketoption::{error::PocketOptionError, utils::connect::try_connect};
use binary_options_tools_core::{
    error::BinaryOptionsResult,
    general::traits::Connect,
    reimports::{MaybeTlsStream, WebSocketStream},
};

use super::ssid::Ssid;

#[derive(Clone)]
pub struct PocketConnect;

#[async_trait]
impl Connect for PocketConnect {
    type Creds = Ssid;

    async fn connect(
        &self,
        creds: Self::Creds,
    ) -> BinaryOptionsResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let urls = creds.servers().await?;
        let mut error = None;
        for url in urls.clone() {
            match try_connect(creds.clone(), url).await {
                Ok(connect) => return Ok(connect),
                Err(e) => {
                    warn!("Failed to connect to server, {e}");
                    error = Some(e);
                }
            }
        }
        if let Some(error) = error {
            Err(error.into())
        } else {
            Err(
                PocketOptionError::WebsocketMultipleAttemptsConnectionError(format!(
                    "Couldn't connect to server after {} attempts.",
                    urls.len()
                ))
                .into(),
            )
        }
    }
}
