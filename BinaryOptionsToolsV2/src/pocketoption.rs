use std::sync::Arc;

use binary_option_tools_core::pocketoption::ws::listener::Handler;
use binary_option_tools_core::pocketoption::WebSocketClient;

use pyo3::{pyclass, pyfunction, pymethods, Bound, IntoPy, PyAny, PyResult, Python};
use pyo3_asyncio_0_21::tokio::future_into_py;
use uuid::Uuid;

use crate::error::BinaryErrorPy;

#[pyclass]
#[derive(Clone)]
pub struct RawPocketOption {
    client: Arc<WebSocketClient<Handler>>,
}

#[pyfunction]
pub fn connect(py: Python, ssid: String) -> PyResult<Bound<PyAny>> {
    future_into_py(py, async move {
        let client = WebSocketClient::<Handler>::new(ssid)
            .await
            .map_err(BinaryErrorPy::from)?;
        let pocket_option = RawPocketOption {
            client: Arc::new(client),
        };
        Python::with_gil(|py: Python<'_>| Ok(pocket_option.into_py(py)))
    })
}

#[pymethods]
impl RawPocketOption {
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
}
