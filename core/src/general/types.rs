use std::{collections::HashMap, ops::Deref, sync::Arc};

use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Sender as OneShotSender;
use async_channel::Sender;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, warn};

use crate::error::BinaryOptionsResult;
use crate::error::BinaryOptionsToolsError;

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

pub struct DropLogger;

pub struct OneShotWrapper<Transfer>
where
    Transfer: MessageTransfer,
{
    inner: OneShotSender<Transfer>,
    _dropper: DropLogger,
}

impl<Transfer> Deref for OneShotWrapper<Transfer>
where
    Transfer: MessageTransfer,
{
    type Target = OneShotSender<Transfer>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
// TODO: Test why the thing doesn't work

impl Drop for DropLogger {
    fn drop(&mut self) {
        warn!("Value dropped")
    }
}

impl<Transfer> From<OneShotWrapper<Transfer>> for OneShotSender<Transfer>
where
    Transfer: MessageTransfer,
{
    fn from(val: OneShotWrapper<Transfer>) -> Self {
        val.inner
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
    pub pending_requests: Arc<
        Mutex<
            HashMap<
                Transfer::Info,
                Vec<(
                    Box<dyn Fn(&Transfer) -> bool + Send + Sync>,
                    OneShotWrapper<Transfer>,
                )>,
            >,
        >,
    >,
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

impl<T, Transfer> Data<T, Transfer>
where
    Transfer: MessageTransfer,
    T: DataHandler,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(inner),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_user_request(
        &self,
        info: Transfer::Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync + 'static,
        sender: OneShotSender<Transfer>,
    ) {
        async fn user<T: DataHandler, Transfer: MessageTransfer>(
            data: &Data<T, Transfer>,
            info: Transfer::Info,
            validator: impl Fn(&Transfer) -> bool + Send + Sync + 'static,
            sender: OneShotSender<Transfer>,
        ) {
            let mut requests = data.pending_requests.lock().await;
            requests.entry(info).or_default().push((
                Box::new(validator),
                OneShotWrapper {
                    inner: sender,
                    _dropper: DropLogger,
                },
            ));
        }
        user(self, info, validator, sender).await;
        info!("Test");
        // if let Some(reqs) = requests.get_mut(&info) {
        //     reqs.push((Box::new(validator), sender));
        //     return;
        // }
        // info!("Added successfully user request!");
        // requests.insert(info, vec![(Box::new(validator), sender)]);
        // info!("Inserted value to requests");
    }

    pub async fn get_request(
        &self,
        message: &Transfer,
    ) -> BinaryOptionsResult<Option<Vec<OneShotSender<Transfer>>>> {
        let mut requests = self.pending_requests.lock().await;
        let info = message.info();

        if let Some(reqs) = requests.get_mut(&info) {
            // Find the index of the matching validator
            info!("Foun request with correct info type");
            let mut senders = Vec::new();
            let mut keepers = Vec::new();
            let drain = reqs.drain(..);
            drain.for_each(|req| {
                if req.0(message) {
                    senders.push(req);
                } else {
                    keepers.push(req);
                }
            });
            *reqs = keepers;
            if !senders.is_empty() {
                return Ok(Some(
                    senders
                        .into_iter()
                        .map(|(_, s)| s.into())
                        .collect::<Vec<OneShotSender<Transfer>>>(),
                ));
            } else {
                return Ok(None);
            }
        }
        if let Some(error) = message.error() {
            let error = error.into();
            if let Some(reqs) = requests.remove(&info) {
                for (_, sender) in reqs.into_iter() {
                    let sender: OneShotSender<Transfer> = sender.into();
                    sender.send(error.clone())?;
                }
            }
        }
        Ok(None)
    }

    pub async fn list_pending_requests(&self) {
        let requests = self.pending_requests.lock().await;
        requests.iter().for_each(|(k, v)| {
            println!("Request type: {}, amount: {}", k, v.len());
        });
    }
}
/*

#[async_trait]
impl<T, Transfer, Info> DataHandler for Data<T, Transfer, Info>
where
    Transfer: MessageTransfer,
    Info: for<'de> MessageInformation<'de>,
    T: DataHandler
{
    async fn update<M>(&self, message: M, sender: &Sender<Message>)
    where
        M: MessageTransfer
    {
        if message.is_user_request() {
            self.add_user_request(info, validator, sender)
        }
    }
}
*/
impl<T, Transfer> Data<T, Transfer>
where
    Transfer: MessageTransfer + 'static,
    T: DataHandler<Transfer = Transfer>,
{
    pub async fn update_data(
        &self,
        message: Transfer,
        sender: &Sender<Message>,
    ) -> BinaryOptionsResult<()> {
        if let Some(request) = message.user_request() {
            debug!(
                "Recieved user request, the message type is '{}'",
                request.info
            );
            self.add_user_request(request.info, request.validator, request.sender)
                .await;
            warn!("Added user to request");
            let message = *request.message;
            if let Err(e) = sender.send(message.into()).await {
                warn!(
                    "Error sending message: {}",
                    BinaryOptionsToolsError::from(e)
                );
            }
            warn!("Added user to request p2");
        } else {
            self.update(&message).await;
            debug!("Updated data!");
            if let Some(senders) = self.get_request(&message).await? {
                info!("'get_request' function returned some senders, sending message");
                for s in senders {
                    s.send(message.clone())?;
                }
            }
        }
        Ok(())
    }
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
