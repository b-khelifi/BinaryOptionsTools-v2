#![allow(non_snake_case)]

pub mod error;
pub mod pocketoption;
pub mod runtime;

use pocketoption::RawPocketOption;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "BinaryOptionsToolsV2")]
fn BinaryOptionsTools(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RawPocketOption>()?;
    Ok(())
}
