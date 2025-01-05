#![allow(non_snake_case)]

pub mod error;
pub mod logs;
pub mod pocketoption;
pub mod runtime;

use logs::start_tracing;
use pocketoption::RawPocketOption;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "BinaryOptionsToolsV2")]
fn BinaryOptionsTools(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RawPocketOption>()?;

    m.add_function(wrap_pyfunction!(start_tracing, m)?)?;
    Ok(())
}
