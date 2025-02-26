use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::constants::{MAX_ALLOWED_LOOPS, RECONNECT_CALLBACK, SLEEP_INTERVAL};

use super::{traits::{DataHandler, MessageTransfer}, types::Callback};
use binary_options_tools_macros::Config;


#[derive(Serialize, Deserialize, Config)]
pub struct _Config<T: DataHandler, Transfer: MessageTransfer> {
    pub max_allowed_loops: u32,
    pub sleep_interval: u64,
    #[config(extra(optional))]
    pub default_connection_url: Option<Url>,
    pub reconnect_time: u64,
    #[serde(skip)]
    #[config(extra(iterator = "Callback<T, Transfer>"))]
    pub callbacks: Vec<Callback<T, Transfer>>,
    pub timeout: Duration,
    // #[serde(skip)]
    // pub callbacks: Arc<Vec<Arc<dyn Callback>>>
}

impl<T: DataHandler, Transfer: MessageTransfer> _Config<T, Transfer> {
    pub fn new(timeout: Duration, callbacks: Vec<Callback<T, Transfer>>) -> Self {
        Self {
            max_allowed_loops: MAX_ALLOWED_LOOPS,
            sleep_interval: SLEEP_INTERVAL,
            default_connection_url: None,
            reconnect_time: RECONNECT_CALLBACK,
            callbacks,
            timeout
        }
    }
}
