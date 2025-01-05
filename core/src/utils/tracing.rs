use std::fs::OpenOptions;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn start_tracing() -> anyhow::Result<()> {
    let error_logs = OpenOptions::new()
        .append(true)
        .create(true)
        .open("errors.log")?;

    tracing_subscriber::registry()
        // .with(filtered_layer)
        .with(
            // log-error file, to log the errors that arise
            fmt::layer()
                .with_ansi(false)
                .with_writer(error_logs)
                .with_filter(LevelFilter::WARN),
        )
        .with(fmt::Layer::default().with_filter(LevelFilter::DEBUG))
        .try_init()?;
    Ok(())
}
