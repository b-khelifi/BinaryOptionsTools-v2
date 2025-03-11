use binary_option_tools::error::BinaryOptionsResult;
use binary_option_tools::pocketoption::error::PocketResult;
use binary_option_tools::pocketoption::pocket_client::PocketOption;
use binary_option_tools::pocketoption::types::base::RawWebsocketMessage;
use binary_option_tools::pocketoption::types::update::DataCandle;
use binary_option_tools::pocketoption::ws::stream::StreamAsset;
use futures_util::stream::{BoxStream, Fuse};
use futures_util::StreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

#[napi(js_name = "PocketOption")]
pub struct RawPocketOption {
  client: PocketOption,
}

#[napi]
pub struct StreamIterator {
    stream: Arc<Mutex<Fuse<BoxStream<'static, PocketResult<DataCandle>>>>>,
}

#[napi]
pub struct RawStreamIterator {
    stream: Arc<Mutex<Fuse<BoxStream<'static, BinaryOptionsResult<RawWebsocketMessage>>>>>,
}

#[napi]
impl RawPocketOption {
  //   #[napi(constructor)]

    pub async fn new(ssid: String) -> Result<Self> {
        let client = PocketOption::new(ssid)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { client })
    }

    #[napi]
    pub async fn new_with_url(ssid: String, url: String) -> Result<Self> {
        let client = PocketOption::new_with_url(
        ssid,
        Url::parse(&url).map_err(|e| Error::from_reason(e.to_string()))?,
        )
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { client })
    }

    #[napi]
    pub async fn buy(&self, asset: String, amount: f64, time: u32) -> Result<Vec<String>> {
        let res = self
        .client
        .buy(asset, amount, time)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        let deal = serde_json::to_string(&res.1).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(vec![res.0.to_string(), deal])
    }

    #[napi]
    pub async fn sell(&self, asset: String, amount: f64, time: u32) -> Result<Vec<String>> {
        let res = self
        .client
        .sell(asset, amount, time)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        let deal = serde_json::to_string(&res.1).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(vec![res.0.to_string(), deal])
    }

    #[napi]
    pub async fn check_win(&self, trade_id: String) -> Result<String> {
        let res = self
        .client
        .check_results(Uuid::parse_str(&trade_id).map_err(|e| Error::from_reason(e.to_string()))?)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        serde_json::to_string(&res).map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_deal_end_time(&self, trade_id: String) -> Result<Option<i64>> {
        Ok(
        self
            .client
            .get_deal_end_time(
            Uuid::parse_str(&trade_id).map_err(|e| Error::from_reason(e.to_string()))?,
            )
            .await
            .map(|t| t.timestamp()),
        )
    }

    #[napi]
    pub async fn get_candles(&self, asset: String, period: i64, offset: i64) -> Result<String> {
        let res = self
        .client
        .get_candles(asset, period, offset)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        serde_json::to_string(&res).map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn balance(&self) -> Result<String> {
        let res = self.client.get_balance().await;
        serde_json::to_string(&res).map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn closed_deals(&self) -> Result<String> {
        let res = self.client.get_closed_deals().await;
        serde_json::to_string(&res).map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn clear_closed_deals(&self) {
        self.client.clear_closed_deals().await
    }

    #[napi]
    pub async fn opened_deals(&self) -> Result<String> {
        let res = self.client.get_opened_deals().await;
        serde_json::to_string(&res).map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn payout(&self) -> Result<String> {
        let res = self.client.get_payout().await;
        serde_json::to_string(&res).map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn history(&self, asset: String, period: i64) -> Result<String> {
        let res = self
        .client
        .history(asset, period)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        serde_json::to_string(&res).map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn subscribe_symbol(&self, symbol: String) -> Result<StreamIterator> {
        let stream_asset = self
        .client
        .subscribe_symbol(symbol)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        let boxed_stream = StreamAsset::to_stream_static(Arc::new(stream_asset))
        .boxed()
        .fuse();
        let stream = Arc::new(Mutex::new(boxed_stream));
        Ok(StreamIterator { stream })
    }

    #[napi]
    pub async fn subscribe_symbol_chunked(
        &self,
        symbol: String,
        chunk_size: u32,
    ) -> Result<StreamIterator> {
        let stream_asset = self
        .client
        .subscribe_symbol_chuncked(symbol, chunk_size as usize)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        let boxed_stream = StreamAsset::to_stream_static(Arc::new(stream_asset))
        .boxed()
        .fuse();
        let stream = Arc::new(Mutex::new(boxed_stream));
        Ok(StreamIterator { stream })
    }

    #[napi]
    pub async fn subscribe_symbol_timed(
        &self,
        symbol: String,
        time_seconds: u32,
    ) -> Result<StreamIterator> {
        let stream_asset = self
        .client
        .subscribe_symbol_timed(symbol, Duration::from_secs(time_seconds as u64))
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
        let boxed_stream = StreamAsset::to_stream_static(Arc::new(stream_asset))
        .boxed()
        .fuse();
        let stream = Arc::new(Mutex::new(boxed_stream));
        Ok(StreamIterator { stream })
    }
}
#[napi]
impl StreamIterator {
    #[napi]
    pub async fn next(&self) -> Result<Option<String>> {
        let mut stream = self.stream.lock().await;
        match stream.next().await {
        Some(Ok(candle)) => serde_json::to_string(&candle)
            .map(Some)
            .map_err(|e| Error::from_reason(e.to_string())),
        Some(Err(e)) => Err(Error::from_reason(e.to_string())),
        None => Ok(None),
        }
    }
}
