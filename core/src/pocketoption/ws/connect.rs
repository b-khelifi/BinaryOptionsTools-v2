use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{handshake::client::generate_key, http::Request},
    Connector, MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::{
    error::{BinaryOptionsResult, BinaryOptionsToolsError},
    general::traits::Connect,
};

use super::ssid::PocketCreds;

#[derive(Clone)]
pub struct PocketConnect;

#[async_trait]
impl Connect for PocketConnect {
    type Creds = PocketCreds;

    async fn connect(
        &self,
        creds: PocketCreds,
    ) -> BinaryOptionsResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let tls_connector = native_tls::TlsConnector::builder().build()?;

        let connector = Connector::NativeTls(tls_connector);

        let url = creds.server().await?;
        let user_agent = creds.user_agent();
        let t_url = Url::parse(&url).map_err(|e| {
            BinaryOptionsToolsError::GeneralParsingError(format!("Error getting host, {e}"))
        })?;
        let host = t_url
            .host_str()
            .ok_or(BinaryOptionsToolsError::GeneralParsingError(
                "Host not found".into(),
            ))?;
        let request = Request::builder()
            .uri(url)
            .header("Origin", "https://pocketoption.com")
            .header("Cache-Control", "no-cache")
            .header("User-Agent", user_agent)
            .header("Upgrade", "websocket")
            .header("Connection", "upgrade")
            .header("Sec-Websocket-Key", generate_key())
            .header("Sec-Websocket-Version", "13")
            .header("Host", host)
            .body(())?;

        let (ws, _) = connect_async_tls_with_config(request, None, false, Some(connector)).await?;
        Ok(ws)
    }

    async fn try_connect(
        &self,
        _creds: PocketCreds,
    ) -> BinaryOptionsResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        todo!()
    }
}
