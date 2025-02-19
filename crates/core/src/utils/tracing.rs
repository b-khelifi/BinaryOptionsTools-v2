use std::fs::OpenOptions;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::{constants::MAX_LOGGING_CHANNEL_CAPACITY, general::stream::RecieverStream};

pub fn start_tracing(terminal: bool) -> anyhow::Result<()> {
    let error_logs = OpenOptions::new()
        .append(true)
        .create(true)
        .open("errors.log")?;

    let sub = tracing_subscriber::registry()
        // .with(filtered_layer)
        .with(
            // log-error file, to log the errors that arise
            fmt::layer()
                .with_ansi(false)
                .with_writer(error_logs)
                .with_filter(LevelFilter::WARN),
        );
    if terminal {
        sub.with(fmt::Layer::default().with_filter(LevelFilter::DEBUG))
            .try_init()?;
    } else {
        sub.try_init()?;
    }

    Ok(())
}

pub fn start_tracing_leveled(terminal: bool, level: LevelFilter) -> anyhow::Result<()> {
    let error_logs = OpenOptions::new()
        .append(true)
        .create(true)
        .open("errors.log")?;

    let sub = tracing_subscriber::registry()
        // .with(filtered_layer)
        .with(
            // log-error file, to log the errors that arise
            fmt::layer()
                .with_ansi(false)
                .with_writer(error_logs)
                .with_filter(LevelFilter::WARN),
        );
    if terminal {
        sub.with(fmt::Layer::default().with_filter(level))
            .try_init()?;
    } else {
        sub.try_init()?;
    }

    Ok(())
}

#[allow(unused)]
pub fn make_stream(level: LevelFilter) -> anyhow::Result<()> {
    let (sender, reciever) = async_channel::bounded(MAX_LOGGING_CHANNEL_CAPACITY);
    let reciever: RecieverStream<u8> = RecieverStream::new(reciever);


    Ok(())
}