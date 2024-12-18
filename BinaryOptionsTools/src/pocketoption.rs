use std::sync::Arc;

use binary_option_tools_core::pocketoption::WebSocketClient;
use binary_option_tools_core::pocketoption::ws::listener::Handler;

use pyo3::{pyclass, pyfunction, pymethods, Bound, IntoPy, PyAny, PyResult, Python};
use pyo3_asyncio_0_21::tokio::future_into_py;
use uuid::Uuid;

use crate::error::BinaryErrorPy;

#[pyclass]
#[derive(Clone)]
pub struct RawPocketOption {
    client: Arc<WebSocketClient<Handler>>
}

#[pyfunction]
pub fn connect(py: Python, ssid: String, demo: bool) -> PyResult<Bound<PyAny>> {
    future_into_py(py, async move {
        let client = WebSocketClient::<Handler>::new(ssid, demo).await.map_err(BinaryErrorPy::from)?;
        let pocket_option = RawPocketOption { client: Arc::new(client )};
        Python::with_gil(|py: Python<'_>| Ok(pocket_option.into_py(py)))
    })
}
// pub fn validate_token<'py>(&self, py: Python<'py>, token: String) -> PyResult<Bound<'py, PyAny>> {
//     let url = format!("{}{VALIDATE}{token}", self.base_url);
//     let this = self.clone();
//     future_into_py(py, async move {
//         let res = this._send_secure::<Value>(url, None, Method::Post, generate_temporal_id()?).await?;
//         Python::with_gil(|py: Python<'_>| Ok(res.into_py(py)))
//     })
// }


#[pymethods]
impl RawPocketOption {
    pub fn buy<'py>(&self, py: Python<'py>, asset: String, amount: f64, time: u32) -> PyResult<Bound<'py, PyAny>> { //
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.buy(asset, amount, time).await.map_err(BinaryErrorPy::from)?;
            let deal = serde_json::to_string(&res.1).map_err(BinaryErrorPy::from)?;
            let result = (res.0.to_string(), deal);
            Ok(result)
        })
    }

    pub fn sell<'py>(&self, py: Python<'py>, asset: String, amount: f64, time: u32) -> PyResult<Bound<'py, PyAny>> { //
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.sell(asset, amount, time).await.map_err(BinaryErrorPy::from)?;
            let deal = serde_json::to_string(&res.1).map_err(BinaryErrorPy::from)?;
            let result = (res.0.to_string(), deal);
            Ok(result)
        })
    }

    pub fn check_win<'py>(&self, py: Python<'py>, trade_id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.check_results(Uuid::parse_str(&trade_id).map_err(BinaryErrorPy::from)?).await.map_err(BinaryErrorPy::from)?;
            Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
        })
    }

    pub fn get_candles<'py>(&self, py: Python<'py>, asset: String, period: i64, offset: i64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.get_candles(asset, period, offset).await.map_err(BinaryErrorPy::from)?;
            Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
        })
    }

    pub fn balance<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.get_balande().await;
            Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
        })
    }

    pub fn closed_deals<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.get_closed_deals().await;
            Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
        })
    }

    pub fn opened_deals<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.get_opened_deals().await;
            Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
        })
    }

    pub fn payout<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let res = client.get_payout().await;
            Ok(serde_json::to_string(&res).map_err(BinaryErrorPy::from)?)
        })
    }
}