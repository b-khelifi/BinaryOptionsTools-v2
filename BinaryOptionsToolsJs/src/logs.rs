use std::{fs::OpenOptions, io::Write, sync::Arc};

use binary_options_tools::{
    error::BinaryOptionsResult,
    stream::{stream_logs_layer, RecieverStream},
};
use chrono::Duration;
use futures_util::{
    stream::{BoxStream, Fuse},
    StreamExt,
};
use napi::{Error, Result};
use napi_derive::napi;
use tokio::sync::Mutex;
use tracing::{debug, error, info, level_filters::LevelFilter, warn, Level};
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer, Registry,
};

const TARGET: &str = "JavaScript";

/// Initializes the logging system with the specified configuration.
///
/// # Arguments
/// * `path` - Directory path where log files will be stored
/// * `level` - Logging level ("DEBUG", "INFO", "WARN", "ERROR")
/// * `terminal` - Whether to output logs to terminal
/// * `layers` - Additional logging layers for custom log handling
///
/// # Examples
/// ```javascript
/// const { startTracing } = require('binary-options-tools');
///
/// // Initialize logging with DEBUG level and terminal output
/// startTracing('./logs', 'DEBUG', true, []);
/// ```
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

/// A custom logging layer that can be used to capture and process log messages.
/// Used in conjunction with `StreamLogsIterator` to receive log messages.
///
/// # Examples
/// ```javascript
/// const { StreamLogsLayer, LogBuilder } = require('binary-options-tools');
///
/// const builder = new LogBuilder();
/// const iterator = builder.createLogsIterator('DEBUG');
/// ```
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

/// Iterator for receiving log messages from a `StreamLogsLayer`.
/// Supports asynchronous iteration over log messages.
///
/// # Examples
/// ```javascript
/// const iterator = builder.createLogsIterator('DEBUG');
///
/// // Async iteration
/// for await (const log of iterator) {
///     console.log('Received log:', log);
/// }
/// ```
#[napi]
pub struct StreamLogsIterator {
    stream: Arc<Mutex<LogStream>>,
}

#[napi]
impl StreamLogsIterator {
    /// Gets the next log message from the stream.
    ///
    /// # Returns
    /// * A Promise that resolves to:
    ///   * A string containing the next log message
    ///   * null if the stream has ended
    /// * Rejects with an error if the stream encounters an error
    ///
    /// # Examples
    /// ```javascript
    /// const iterator = builder.createLogsIterator('DEBUG');
    ///
    /// // Using async/await
    /// try {
    ///     while (true) {
    ///         const log = await iterator.next();
    ///         if (log === null) break;
    ///         console.log('Log:', log);
    ///     }
    /// } catch (err) {
    ///     console.error('Stream error:', err);
    /// }
    /// ```
    #[napi]
    pub async fn next(&self) -> Result<Option<serde_json::Value>> {
        let mut stream = self.stream.lock().await;
        match stream.next().await {
            Some(Ok(msg)) => Ok(Some(serde_json::from_str(&msg)?)),
            Some(Err(e)) => Err(Error::from_reason(e.to_string())),
            None => Ok(None),
        }
    }
}

/// Builder pattern for configuring the logging system.
/// Allows adding multiple log outputs and configuring log levels.
///
/// # Examples
/// ```javascript
/// const { LogBuilder } = require('binary-options-tools');
///
/// const builder = new LogBuilder();
/// // Add file logging
/// builder.logFile('./app.log', 'INFO');
/// // Add terminal logging
/// builder.terminal('DEBUG');
/// // Create log stream
/// const iterator = builder.createLogsIterator('DEBUG');
/// // Initialize logging
/// builder.build();
/// ```
#[napi]
#[derive(Default)]
pub struct LogBuilder {
    layers: Vec<Box<dyn Layer<Registry> + Send + Sync>>,
    build: bool,
}

#[napi]
impl LogBuilder {
    /// Creates a new LogBuilder instance.
    ///
    /// # Examples
    /// ```javascript
    /// const { LogBuilder } = require('binary-options-tools');
    /// const builder = new LogBuilder();
    /// ```
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new logs iterator that receives log messages at the specified level.
    ///
    /// # Arguments
    /// * `level` - Logging level ("DEBUG", "INFO", "WARN", "ERROR")
    /// * `timeout` - Optional timeout in seconds after which the iterator will stop
    ///
    /// # Returns
    /// * A StreamLogsIterator that can be used to receive log messages
    ///
    /// # Examples
    /// ```javascript
    /// const builder = new LogBuilder();
    ///
    /// // Create iterator with default DEBUG level
    /// const iterator1 = builder.createLogsIterator('DEBUG');
    ///
    /// // Create iterator with INFO level and 60-second timeout
    /// const iterator2 = builder.createLogsIterator('INFO', 60);
    /// ```
    #[napi]
    pub fn create_logs_iterator(
        &mut self,
        level: String,
        timeout: Option<i64>,
    ) -> Result<StreamLogsIterator> {
        let timeout = timeout.map(Duration::seconds).and_then(|d| d.to_std().ok());

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

    /// Adds a file output for logs at the specified level.
    ///
    /// # Arguments
    /// * `path` - Path to the log file
    /// * `level` - Logging level ("DEBUG", "INFO", "WARN", "ERROR")
    ///
    /// # Returns
    /// * Result indicating success or failure
    ///
    /// # Examples
    /// ```javascript
    /// const builder = new LogBuilder();
    ///
    /// // Log INFO and above to app.log
    /// builder.logFile('./app.log', 'INFO');
    ///
    /// // Log DEBUG and above to debug.log
    /// builder.logFile('./debug.log', 'DEBUG');
    /// ```
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

    /// Adds terminal (console) output for logs at the specified level.
    ///
    /// # Arguments
    /// * `level` - Logging level ("DEBUG", "INFO", "WARN", "ERROR")
    ///
    /// # Examples
    /// ```javascript
    /// const builder = new LogBuilder();
    ///
    /// // Show DEBUG and above in terminal
    /// builder.terminal('DEBUG');
    ///
    /// // Show only INFO and above in terminal
    /// builder.terminal('INFO');
    /// ```
    #[napi]
    pub fn terminal(&mut self, level: String) {
        let layer = fmt::Layer::default()
            .with_filter(level.parse().unwrap_or(LevelFilter::DEBUG))
            .boxed();
        self.layers.push(layer);
    }

    /// Finalizes the logging configuration and initializes the logging system.
    /// Must be called after all outputs are configured and before logging begins.
    /// Can only be called once per builder instance.
    ///
    /// # Returns
    /// * Result indicating success or failure
    ///
    /// # Examples
    /// ```javascript
    /// const builder = new LogBuilder();
    /// builder.logFile('./app.log', 'INFO');
    /// builder.terminal('DEBUG');
    ///
    /// // Initialize logging system
    /// builder.build();
    ///
    /// // Attempting to build again will result in an error
    /// builder.build(); // Error: Builder has already been built
    /// ```
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

/// Simple logging interface for emitting log messages at different levels.
///
/// # Examples
/// ```javascript
/// const { Logger } = require('binary-options-tools');
///
/// const logger = new Logger();
/// logger.debug('Debug message');
/// logger.info('Info message');
/// logger.warn('Warning message');
/// logger.error('Error message');
/// ```
#[napi]
#[derive(Default)]
pub struct Logger;

#[napi]
impl Logger {
    /// Creates a new Logger instance.
    ///
    /// # Examples
    /// ```javascript
    /// const { Logger } = require('binary-options-tools');
    /// const logger = new Logger();
    /// ```
    #[napi(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// Logs a debug message.
    /// Only appears if logging level is set to DEBUG.
    ///
    /// # Arguments
    /// * `message` - The message to log
    ///
    /// # Examples
    /// ```javascript
    /// const logger = new Logger();
    /// logger.debug('Processing started');
    /// logger.debug(`Current value: ${value}`);
    /// ```
    #[napi]
    pub fn debug(&self, message: String) {
        debug!(target: TARGET, message);
    }

    /// Logs an info message.
    /// Only appears if logging level is set to INFO or lower.
    ///
    /// # Arguments
    /// * `message` - The message to log
    ///
    /// # Examples
    /// ```javascript
    /// const logger = new Logger();
    /// logger.info('Operation completed successfully');
    /// logger.info(`Processed ${count} items`);
    /// ```
    #[napi]
    pub fn info(&self, message: String) {
        info!(target: TARGET, message);
    }

    /// Logs a warning message.
    /// Only appears if logging level is set to WARN or lower.
    ///
    /// # Arguments
    /// * `message` - The message to log
    ///
    /// # Examples
    /// ```javascript
    /// const logger = new Logger();
    /// logger.warn('Resource usage high');
    /// logger.warn(`Retry attempt ${retryCount} of ${maxRetries}`);
    /// ```
    #[napi]
    pub fn warn(&self, message: String) {
        warn!(target: TARGET, message);
    }

    /// Logs an error message.
    /// Only appears if logging level is set to ERROR or lower.
    ///
    /// # Arguments
    /// * `message` - The message to log
    ///
    /// # Examples
    /// ```javascript
    /// const logger = new Logger();
    /// logger.error('Operation failed');
    /// logger.error(`Failed to connect: ${error.message}`);
    /// ```
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
