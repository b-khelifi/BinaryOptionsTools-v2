use async_trait::async_trait;
use tokio::net::TcpStream;
use tracing::{info, warn};

use crate::pocketoption::{error::PocketOptionError, utils::connect::try_connect};
use binary_options_tools_core::{
    error::BinaryOptionsResult,
    general::{
        config::Config,
        traits::{Connect, DataHandler, MessageTransfer},
    },
    reimports::{MaybeTlsStream, WebSocketStream},
};

use super::ssid::Ssid;

#[derive(Clone)]
pub struct PocketConnect;

#[async_trait]
impl Connect for PocketConnect {
    type Creds = Ssid;

    async fn connect<T: DataHandler, Transfer: MessageTransfer>(
        &self,
        creds: Self::Creds,
        config: &Config<T, Transfer>,
    ) -> BinaryOptionsResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        if let Some(url) = config.get_default_connection_url()? {
            info!("Using default connection url...");
            if let Ok(connect) = try_connect(creds.clone(), url.to_string()).await {
                return Ok(connect);
            }
        }
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
