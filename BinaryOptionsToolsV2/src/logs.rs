use std::fs::OpenOptions;

use pyo3::{pyclass, pyfunction, pymethods, PyResult};
use tracing::{debug, instrument, level_filters::LevelFilter, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

const TARGET: &str = "Python";

#[pyfunction]
pub fn start_tracing(path: String, level: String, terminal: bool) -> PyResult<()> {
    let level: LevelFilter = level.parse().unwrap_or(Level::DEBUG.into());
    let error_logs = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/error.log", &path))?;
    let logs = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/logs.log", &path))?;
    let subscriber = tracing_subscriber::registry()
        // .with(filtered_layer)
        .with(
            // log-error file, to log the errors that arise
            fmt::layer()
                .with_ansi(false)
                .with_writer(error_logs)
                .with_filter(LevelFilter::WARN),
        )
        .with(
            // log-debug file, to log the debug
            fmt::layer()
                .with_ansi(false)
                .with_writer(logs)
                .with_filter(level),
        );
    if terminal {
        subscriber
            .with(fmt::Layer::default().with_filter(level))
            .with(fmt::Layer::default().with_filter(level))
            .init();
    } else {
        subscriber.init()
    }

    Ok(())
}

#[pyclass]
#[derive(Default)]
pub struct Logger;

#[pymethods]
impl Logger {
    #[new]
    pub fn new() -> Self {
        Self
    }

    #[instrument(target = TARGET, skip(self, data))] // Use instrument for better tracing
    pub fn debug(&self, data: String) {
        debug!(data);
    }

    #[instrument(target = TARGET, skip(self, data))]
    pub fn info(&self, data: String) {
        tracing::info!(data);
    }

    #[instrument(target = TARGET, skip(self, data))]
    pub fn warn(&self, data: String) {
        tracing::warn!(data);
    }

    #[instrument(target = TARGET, skip(self, data))]
    pub fn error(&self, data: String) {
        tracing::error!(data);
    }
}
