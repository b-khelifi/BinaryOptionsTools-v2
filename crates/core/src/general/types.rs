use std::{collections::HashMap, ops::Deref, sync::Arc};

use async_channel::bounded;
use async_channel::Receiver;
use async_channel::Sender;
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::constants::MAX_CHANNEL_CAPACITY;
use crate::error::BinaryOptionsResult;

use super::send::SenderMessage;
use super::traits::WCallback;
use super::traits::{DataHandler, MessageTransfer};

#[derive(Clone)]
pub enum MessageType<Transfer>
where
    Transfer: MessageTransfer,
{
    Info(Transfer::Info),
    Transfer(Transfer),
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


#[derive(Clone)]
pub struct Callback<T: DataHandler, Transfer: MessageTransfer> {
    inner: Arc<dyn WCallback<T = T, Transfer = Transfer>>,
}

pub fn default_validator<Transfer: MessageTransfer>(_val: &Transfer) -> bool {
    false
}

impl<T: DataHandler, Transfer: MessageTransfer> Callback<T, Transfer> {
    pub fn new(callback: Arc<dyn WCallback<T = T, Transfer = Transfer>>) -> Self {
        Self { inner: callback }
    }
}

#[async_trait]
impl<T: DataHandler, Transfer: MessageTransfer> WCallback for Callback<T, Transfer> {
    type T = T;
    type Transfer = Transfer;

    async fn call(
        &self,
        data: Data<Self::T, Self::Transfer>,
        sender: &SenderMessage,
    ) -> BinaryOptionsResult<()> {
        self.inner.call(data, sender).await
    }
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
