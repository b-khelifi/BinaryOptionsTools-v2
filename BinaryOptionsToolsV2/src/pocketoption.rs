use std::str;
use std::sync::Arc;

use binary_option_tools_core::pocketoption::error::PocketResult;
use binary_option_tools_core::pocketoption::ws::stream::StreamAsset;
use futures_util::StreamExt;
use binary_option_tools_core::pocketoption::{types::update::DataCandle, ws::listener::Handler};
use binary_option_tools_core::pocketoption::WebSocketClient;
use futures_util::stream::{BoxStream, Fuse};
use pyo3::exceptions::PyStopIteration;
use pyo3::{pyclass, pymethods, Bound, Py, PyAny, PyResult, Python};
use pyo3_async_runtimes::tokio::future_into_py;
use uuid::Uuid;

use tokio::sync::Mutex;
use crate::error::BinaryErrorPy;
use crate::runtime::get_runtime;



#[pyclass]
#[derive(Clone)]
pub struct RawPocketOption {
    client: Arc<WebSocketClient<Handler>>,
}

#[pyclass]
pub struct StreamIterator {
    stream: Arc<Mutex<Fuse<BoxStream<'static, PocketResult<DataCandle>>>>>
}



#[pymethods]
impl RawPocketOption {
    #[new]
    pub fn new(ssid: String, py: Python<'_>) -> PyResult<Self> {
        let runtime = get_runtime(py)?;
        runtime.block_on(async move { 
            let client = WebSocketClient::<Handler>::new(ssid).await.map_err(BinaryErrorPy::from)?;
            Ok(Self{client:Arc::new(client)})
        })
    }

    pub async fn buy(&self, asset: String, amount: f64, time: u32) -> PyResult<Vec<String>> {
        let res = self
            .client
            .buy(asset, amount, time)
            .await
            .map_err(BinaryErrorPy::from)?;
        let deal = serde_json::to_string(&res.1).map_err(BinaryErrorPy::from)?;
        let result = vec![res.0.to_string(), deal];
        Ok(result)
    }

    pub async fn sell(&self, asset: String, amount: f64, time: u32) -> PyResult<Vec<String>> {
        let res = self
            .client
            .sell(asset, amount, time)
            .await
            .map_err(BinaryErrorPy::from)?;
        let deal = serde_json::to_string(&res.1).map_err(BinaryErrorPy::from)?;
        let result = vec![res.0.to_string(), deal];
        Ok(result)
    }

    pub async fn check_win(&self, trade_id: String) -> PyResult<String> {
        let res = self
            .client
            .check_results(Uuid::parse_str(&trade_id).map_err(BinaryErrorPy::from)?)
            .await
            .map_err(BinaryErrorPy::from)?;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn get_candles(&self, asset: String, period: i64, offset: i64) -> PyResult<String> {
        let res = self
            .client
            .get_candles(asset, period, offset)
            .await
            .map_err(BinaryErrorPy::from)?;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn balance(&self) -> PyResult<String> {
        let res = self.client.get_balande().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn closed_deals(&self) -> PyResult<String> {
        let res = self.client.get_closed_deals().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn opened_deals(&self) -> PyResult<String> {
        let res = self.client.get_opened_deals().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn payout(&self) -> PyResult<String> {
        let res = self.client.get_payout().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn history(&self, asset: String, period: i64) -> PyResult<String> {
        let res = self.client.history(asset, period).await.map_err(BinaryErrorPy::from)?;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn subscribe_symbol(&self, symbol: String) -> PyResult<StreamIterator> {
        let stream_asset = self.client.subscribe_symbol(symbol).await.map_err(BinaryErrorPy::from)?;
    
        // Clone the stream_asset and convert it to a BoxStream
        let boxed_stream = StreamAsset::to_stream_static(Arc::new(stream_asset)).boxed().fuse();
        
        // Wrap the BoxStream in an Arc and Mutex
        let stream = Arc::new(Mutex::new(boxed_stream));
        
        Ok(StreamIterator { stream })
    }
}


#[pymethods]
impl StreamIterator {
    fn __aiter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __anext__<'py>(&'py mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let stream = self.stream.clone();
        future_into_py(py, 
            next_stream(stream)
        )
    }

    fn __next__<'py>(&'py self, py: Python<'py>) -> PyResult<String> {
        let runtime = get_runtime(py)?;
        let stream = self.stream.clone();
        runtime.block_on(next_stream(stream))
    }

}

async fn next_stream(stream: Arc<Mutex<Fuse<BoxStream<'static, PocketResult<DataCandle>>>>>) -> PyResult<String> {
    let mut stream = stream.lock().await;
    match stream.next().await {
        Some(item) => match item {
            Ok(itm) => Ok(itm.to_string()),
            Err(e) => Err(PyStopIteration::new_err(e.to_string()))
        },
        None => Err(PyStopIteration::new_err("stream exhausted"))
    }
}
