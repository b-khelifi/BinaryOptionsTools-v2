use std::{fs::OpenOptions, io::Write, sync::Arc};

use binary_option_tools::{
    error::BinaryOptionsResult,
    stream::{stream_logs_layer, RecieverStream},
};
use chrono::Duration;
use futures_util::{
    stream::{BoxStream, Fuse},
    StreamExt,
};
use napi::{
    Error,
    Result,
};
use napi_derive::napi;
use tokio::sync::Mutex;
use tracing::{debug, info, warn, error, Level, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer, Registry,
};

const TARGET: &str = "JavaScript";

#[napi]
pub fn start_tracing(
    path: String,
    level: String,
    terminal: bool,
    layers: Vec<&StreamLogsLayer>,
) -> Result<()> {
    let level: LevelFilter = level.parse().unwrap_or(Level::DEBUG.into());
    let error_logs = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/error.log", &path))
        .map_err(|e| Error::from_reason(e.to_string()))?;
    let logs = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/logs.log", &path))
        .map_err(|e| Error::from_reason(e.to_string()))?;
    let default = fmt::Layer::default().with_writer(NoneWriter).boxed();
    let mut layers = layers
        .into_iter()
        .flat_map(|l| Arc::try_unwrap(l.layer.clone()))
        .collect::<Vec<Box<dyn Layer<Registry> + Send + Sync>>>();
    layers.push(default);
    println!("Length of layers: {}", layers.len());
    let subscriber = tracing_subscriber::registry()
        .with(layers)
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(error_logs)
                .with_filter(LevelFilter::WARN),
        )
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(logs)
                .with_filter(level),
        );

    if terminal {
        subscriber
            .with(fmt::Layer::default().with_filter(level))
            .init();
    } else {
        subscriber.init()
    }

    Ok(())
}

#[napi]
#[derive(Clone)]
pub struct StreamLogsLayer {
    layer: Arc<Box<dyn Layer<Registry> + Send + Sync>>,
}

struct NoneWriter;

impl Write for NoneWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for NoneWriter {
    type Writer = NoneWriter;
    fn make_writer(&'a self) -> Self::Writer {
        NoneWriter
    }
}

type LogStream = Fuse<BoxStream<'static, BinaryOptionsResult<String>>>;

#[napi]
pub struct StreamLogsIterator {
    stream: Arc<Mutex<LogStream>>,
}

#[napi]
impl StreamLogsIterator {
    #[napi]
    pub async fn next(&self) -> Result<Option<String>> {
        let mut stream = self.stream.lock().await;
        match stream.next().await {
            Some(Ok(msg)) => Ok(Some(msg)),
            Some(Err(e)) => Err(Error::from_reason(e.to_string())),
            None => Ok(None),
        }
    }
}

#[napi]
#[derive(Default)]
pub struct LogBuilder {
    layers: Vec<Box<dyn Layer<Registry> + Send + Sync>>,
    build: bool,
}

#[napi]
impl LogBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[napi]
    pub fn create_logs_iterator(
        &mut self,
        level: String,
        timeout: Option<i64>,
    ) -> Result<StreamLogsIterator> {
        let timeout = timeout.map(Duration::seconds)
            .and_then(|d| d.to_std().ok());
        
        let (layer, inner_iter) =
            stream_logs_layer(level.parse().unwrap_or(Level::DEBUG.into()), timeout);
        let stream = RecieverStream::to_stream_static(Arc::new(inner_iter))
            .boxed()
            .fuse();
        let iter = StreamLogsIterator {
            stream: Arc::new(Mutex::new(stream)),
        };
        self.layers.push(layer);
        Ok(iter)
    }

    #[napi]
    pub fn log_file(&mut self, path: String, level: String) -> Result<()> {
        let logs = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let layer = fmt::layer()
            .with_ansi(false)
            .with_writer(logs)
            .with_filter(level.parse().unwrap_or(LevelFilter::DEBUG))
            .boxed();
        self.layers.push(layer);
        Ok(())
    }

    #[napi]
    pub fn terminal(&mut self, level: String) {
        let layer = fmt::Layer::default()
            .with_filter(level.parse().unwrap_or(LevelFilter::DEBUG))
            .boxed();
        self.layers.push(layer);
    }

    #[napi]
    pub fn build(&mut self) -> Result<()> {
        if self.build {
            return Err(Error::from_reason(
                "Builder has already been built, cannot be called again".to_string(),
            ));
        }
        self.build = true;
        let default = fmt::Layer::default().with_writer(NoneWriter).boxed();
        self.layers.push(default);
        let layers = self
            .layers
            .drain(..)
            .collect::<Vec<Box<dyn Layer<Registry> + Send + Sync>>>();
        tracing_subscriber::registry().with(layers).init();
        Ok(())
    }
}

#[napi]
#[derive(Default)]
pub struct Logger;

#[napi]
impl Logger {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[napi]
    pub fn debug(&self, message: String) {
        debug!(target: TARGET, message);
    }

    #[napi]
    pub fn info(&self, message: String) {
        info!(target: TARGET, message);
    }

    #[napi]
    pub fn warn(&self, message: String) {
        warn!(target: TARGET, message);
    }

    #[napi]
    pub fn error(&self, message: String) {
        error!(target: TARGET, message);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures_util::future::join;
    use serde_json::Value;
    use tracing::{error, info, trace, warn};

    use super::*;

    #[test]
    fn test_start_tracing() {
        start_tracing(".".to_string(), "DEBUG".to_string(), true, vec![]).unwrap();
        info!("Test")
    }

    fn create_logs_iterator_test(level: String) -> (StreamLogsLayer, StreamLogsIterator) {
        let (inner_layer, inner_iter) =
            stream_logs_layer(level.parse().unwrap_or(Level::DEBUG.into()), None);
        let layer = StreamLogsLayer {
            layer: Arc::new(inner_layer),
        };
        let stream = RecieverStream::to_stream_static(Arc::new(inner_iter))
            .boxed()
            .fuse();
        let iter = StreamLogsIterator {
            stream: Arc::new(Mutex::new(stream)),
        };
        (layer, iter)
    }

    #[tokio::test]
    async fn test_start_tracing_stream() {
        let (layer, receiver) = create_logs_iterator_test("ERROR".to_string());
        start_tracing(".".to_string(), "DEBUG".to_string(), false, vec![&layer]).unwrap();

        async fn log() {
            let mut num = 0;
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                num += 1;
                trace!(num, "Test trace");
                debug!(num, "Test debug");
                info!(num, "Test info");
                warn!(num, "Test warning");
                error!(num, "Test error");
            }
        }

        async fn reciever_fn(reciever: StreamLogsIterator) {
            let mut stream = reciever.stream.lock().await;
            while let Some(Ok(value)) = stream.next().await {
                let value: Value = serde_json::from_str(&format!("{:?}", value)).unwrap();
                println!("{}", value);
            }
        }

        join(log(), reciever_fn(receiver)).await;
    }
}