use std::{collections::HashMap, ops::Deref, sync::Arc};

use async_channel::bounded;
use async_channel::Receiver;
use async_channel::Sender;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::oneshot::Sender as OneShotSender;
use tokio::sync::Mutex;

use crate::contstants::MAX_CHANNEL_CAPACITY;
use crate::error::BinaryOptionsResult;

use super::traits::{DataHandler, MessageTransfer};

#[derive(Clone)]
pub enum MessageType<Transfer>
where
    Transfer: MessageTransfer,
{
    Info(Transfer::Info),
    Transfer(Transfer),
}

pub struct UserRequest<Transfer>
where
    Transfer: MessageTransfer,
{
    pub info: Transfer::Info,
    pub message: Box<Transfer>,
    pub validator: Box<dyn Fn(&Transfer) -> bool + Send + Sync>,
    pub sender: OneShotSender<Transfer>,
}

impl<Transfer> UserRequest<Transfer>
where
    Transfer: MessageTransfer,
{
    pub fn new(
        message: Transfer,
        info: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync + 'static,
    ) -> (Self, tokio::sync::oneshot::Receiver<Transfer>) {
        let (sender, reciever) = tokio::sync::oneshot::channel::<Transfer>();
        let request = Self {
            message: Box::new(message),
            info,
            validator: Box::new(validator),
            sender,
        };
        (request, reciever)
    }
}

impl<Transfer> Clone for UserRequest<Transfer>
where
    Transfer: MessageTransfer + 'static,
{
    fn clone(&self) -> Self {
        let (sender, _) = tokio::sync::oneshot::channel();
        Self {
            message: self.message.clone(),
            info: self.info.clone(),
            validator: Box::new(default_validator),
            sender,
        }
    }
}

pub fn default_validator<Transfer: MessageTransfer>(_val: &Transfer) -> bool {
    false
}

impl<'de, Transfer> Deserialize<'de> for UserRequest<Transfer>
where
    Transfer: MessageTransfer + 'static,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let message = serde_json::from_value(
            value
                .get("message")
                .ok_or(serde::de::Error::missing_field("Missing field 'message'"))?
                .clone(),
        )
        .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        let info: Transfer::Info = serde_json::from_value(
            value
                .get("info")
                .ok_or(serde::de::Error::missing_field("Missing field 'info'"))?
                .clone(),
        )
        .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        let (sender, _) = tokio::sync::oneshot::channel::<Transfer>();
        Ok(Self {
            message,
            info,
            validator: Box::new(default_validator),
            sender,
        })
    }
}

#[derive(Default, Clone)]
pub struct Data<T, Transfer>
where
    Transfer: MessageTransfer,
    T: DataHandler,
{
    inner: Arc<T>,
    #[allow(clippy::type_complexity)]
    pub pending_requests:
        Arc<Mutex<HashMap<Transfer::Info, (Sender<Transfer>, Receiver<Transfer>)>>>,
}

impl<T, Transfer> Data<T, Transfer>
where
    Transfer: MessageTransfer,
    T: DataHandler<Transfer = Transfer>,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(inner),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_request(&self, info: Transfer::Info) -> Receiver<Transfer> {
        let mut requests = self.pending_requests.lock().await;
        let (_, r) = requests
            .entry(info)
            .or_insert(bounded(MAX_CHANNEL_CAPACITY));
        r.clone()
    }

    pub async fn sender(&self, info: Transfer::Info) -> Option<Sender<Transfer>> {
        let requests = self.pending_requests.lock().await;
        requests.get(&info).map(|(s, _)| s.clone())
    }

    pub async fn get_sender(&self, message: &Transfer) -> Option<Vec<Sender<Transfer>>> {
        let requests = self.pending_requests.lock().await;
        if let Some(infos) = &message.error_info() {
            return Some(
                infos
                    .iter()
                    .filter_map(|i| requests.get(i).map(|(s, _)| s.to_owned()))
                    .collect(),
            );
        }
        requests
            .get(&message.info())
            .map(|(s, _)| vec![s.to_owned()])
    }

    pub async fn update_data(
        &self,
        message: Transfer,
    ) -> BinaryOptionsResult<Option<Vec<Sender<Transfer>>>> {
        self.inner.update(&message).await?;
        Ok(self.get_sender(&message).await)
    }
}

impl<T, Transfer> Deref for Data<T, Transfer>
where
    Transfer: MessageTransfer,
    T: DataHandler,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
