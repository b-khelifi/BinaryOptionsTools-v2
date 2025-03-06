use binary_options_tools_core::{
    error::BinaryOptionsToolsError,
    reimports::{
        Connector, MaybeTlsStream, Request, WebSocketStream, connect_async_tls_with_config,
        generate_key,
    },
};
use tokio::net::TcpStream;
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
        .uri(t_url.to_string())
        .header("Origin", "https://pocketoption.com")
        .header("Cache-Control", "no-cache")
        .header("User-Agent", user_agent)
        .header("Upgrade", "websocket")
        .header("Connection", "upgrade")
        .header("Sec-Websocket-Key", generate_key())
        .header("Sec-Websocket-Version", "13")
        .header("Host", host)
        .body(())
        .map_err(BinaryOptionsToolsError::from)?;

    let (ws, _) = connect_async_tls_with_config(request, None, false, Some(connector))
        .await
        .map_err(BinaryOptionsToolsError::from)?;
    Ok(ws)
}
