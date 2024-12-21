use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{handshake::client::generate_key, http::Request},
    Connector, MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::pocketoption::{
    error::{PocketOptionError, PocketResult},
    ws::ssid::Ssid,
};

pub async fn try_connect(
    ssid: Ssid,
    url: String,
) -> PocketResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let tls_connector = native_tls::TlsConnector::builder().build()?;

    let connector = Connector::NativeTls(tls_connector);

    let user_agent = ssid.user_agent();
    let t_url = Url::parse(&url)
        .map_err(|e| PocketOptionError::GeneralParsingError(format!("Error getting host, {e}")))?;
    let host = t_url
        .host_str()
        .ok_or(PocketOptionError::GeneralParsingError(
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
