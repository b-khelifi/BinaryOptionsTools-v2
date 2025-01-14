use std::str;
use std::sync::Arc;

use binary_option_tools_core::pocketoption::error::PocketResult;
use binary_option_tools_core::pocketoption::pocket_client::PocketOption;
use binary_option_tools_core::pocketoption::types::update::DataCandle;
use binary_option_tools_core::pocketoption::ws::stream::StreamAsset;
use futures_util::stream::{BoxStream, Fuse};
use futures_util::StreamExt;
use pyo3::exceptions::PyStopIteration;
use pyo3::{pyclass, pymethods, Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python};
use pyo3_async_runtimes::tokio::future_into_py;
use uuid::Uuid;

use crate::error::BinaryErrorPy;
use crate::runtime::get_runtime;
use tokio::sync::Mutex;

#[pyclass]
#[derive(Clone)]
pub struct RawPocketOption {
    client: PocketOption,
}

#[pyclass]
pub struct StreamIterator {
    stream: Arc<Mutex<Fuse<BoxStream<'static, PocketResult<DataCandle>>>>>,
}

#[pymethods]
impl RawPocketOption {
    #[new]
    pub fn new(ssid: String, py: Python<'_>) -> PyResult<Self> {
        let runtime = get_runtime(py)?;
        runtime.block_on(async move {
            let client = PocketOption::new(ssid).await.map_err(BinaryErrorPy::from)?;
            Ok(Self { client })
        })
    }

    pub fn buy<'py>(
        &self,
        py: Python<'py>,
        asset: String,
        amount: f64,
        time: u32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client
                .buy(asset, amount, time)
                .await
                .map_err(BinaryErrorPy::from)?;
            let deal = serde_json::to_string(&res.1).map_err(BinaryErrorPy::from)?;
            let result = vec![res.0.to_string(), deal];
            Python::with_gil(|py| result.into_py_any(py))
        })
    }

    pub fn sell<'py>(
        &self,
        py: Python<'py>,
        asset: String,
        amount: f64,
        time: u32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client
                .sell(asset, amount, time)
                .await
                .map_err(BinaryErrorPy::from)?;
            let deal = serde_json::to_string(&res.1).map_err(BinaryErrorPy::from)?;
            let result = vec![res.0.to_string(), deal];
            Python::with_gil(|py| result.into_py_any(py))
        })
    }

    pub fn check_win<'py>(&self, py: Python<'py>, trade_id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client
                .check_results(Uuid::parse_str(&trade_id).map_err(BinaryErrorPy::from)?)
                .await
                .map_err(BinaryErrorPy::from)?;
            Python::with_gil(|py| {
                serde_json::to_string(&res)
                    .map_err(BinaryErrorPy::from)?
                    .into_py_any(py)
            })
        })
    }

    pub async fn get_deal_end_time(&self, trade_id: String) -> PyResult<Option<i64>> {
        Ok(self
            .client
            .get_deal_end_time(Uuid::parse_str(&trade_id).map_err(BinaryErrorPy::from)?)
            .await
            .map(|d| d.timestamp()))
    }

    pub fn get_candles<'py>(
        &self,
        py: Python<'py>,
        asset: String,
        period: i64,
        offset: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client
                .get_candles(asset, period, offset)
                .await
                .map_err(BinaryErrorPy::from)?;
            Python::with_gil(|py| {
                serde_json::to_string(&res)
                    .map_err(BinaryErrorPy::from)?
                    .into_py_any(py)
            })
        })
    }

    pub async fn balance(&self) -> PyResult<String> {
        let res = self.client.get_balance().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn closed_deals(&self) -> PyResult<String> {
        let res = self.client.get_closed_deals().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn clear_closed_deals(&self) {
        self.client.clear_closed_deals().await
    }

    pub async fn opened_deals(&self) -> PyResult<String> {
        let res = self.client.get_opened_deals().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub async fn payout(&self) -> PyResult<String> {
        let res = self.client.get_payout().await;
        Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
    }

    pub fn history<'py>(
        &self,
        py: Python<'py>,
        asset: String,
        period: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client
                .history(asset, period)
                .await
                .map_err(BinaryErrorPy::from)?;
            Python::with_gil(|py| {
                serde_json::to_string(&res)
                    .map_err(BinaryErrorPy::from)?
                    .into_py_any(py)
            })
        })
    }

    pub fn subscribe_symbol<'py>(&self, py:Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let stream_asset = client
                .subscribe_symbol(symbol)
                .await
                .map_err(BinaryErrorPy::from)?;

            // Clone the stream_asset and convert it to a BoxStream
            let boxed_stream = StreamAsset::to_stream_static(Arc::new(stream_asset))
                .boxed()
                .fuse();

            // Wrap the BoxStream in an Arc and Mutex
            let stream = Arc::new(Mutex::new(boxed_stream));

            Python::with_gil(|py| StreamIterator { stream }.into_py_any(py))
        })
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
        future_into_py(py, next_stream(stream))
    }

    fn __next__<'py>(&'py self, py: Python<'py>) -> PyResult<String> {
        let runtime = get_runtime(py)?;
        let stream = self.stream.clone();
        runtime.block_on(next_stream(stream))
    }
}

async fn next_stream(
    stream: Arc<Mutex<Fuse<BoxStream<'static, PocketResult<DataCandle>>>>>,
) -> PyResult<String> {
    let mut stream = stream.lock().await;
    match stream.next().await {
        Some(item) => match item {
            Ok(itm) => Ok(itm.to_string()),
            Err(e) => Err(PyStopIteration::new_err(e.to_string())),
        },
        None => Err(PyStopIteration::new_err("stream exhausted")),
    }
}
