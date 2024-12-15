use std::{collections::HashMap, ops::Deref, sync::Arc};

use serde::Deserialize;
use serde_json::Value;
use tokio::sync::{mpsc::Sender, oneshot::Sender as OneShotSender};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

use crate::error::BinaryOptionsToolsError;
use crate::error::BinaryOptionsResult;

use super::traits::{DataHandler, MessageInformation, MessageTransfer};

#[derive(Clone)]
pub enum MessageType<Transfer, Info>
where 
    Info: MessageInformation,
    Transfer: MessageTransfer,
{
    Info(Info),
    Transfer(Transfer)
}

pub struct UserRequest<Transfer, Info>
where 
    Transfer: MessageTransfer,
    Info: MessageInformation
{
    pub info: Info,
    pub message: Box<Transfer>,
    pub validator: Box<dyn Fn(&Transfer) -> bool + Send + Sync>,
    pub sender: OneShotSender<Transfer>
}

#[derive(Default, Clone)]
pub struct Data<T, Transfer, Info>
where
    Transfer: MessageTransfer,
    Info: MessageInformation,
    T: DataHandler
{
    inner: Arc<T>,
    pending_requests: Arc<
        Mutex<
            HashMap<
                Info,
                Vec<(
                    Box<dyn Fn(&Transfer) -> bool + Send + Sync>,
                    OneShotSender<Transfer>,
                )>,
            >,
        >,
    >,
}

impl<T, Transfer, Info> Deref for Data<T, Transfer, Info>
where
    Transfer: MessageTransfer,
    Info: MessageInformation,
    T: DataHandler
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, Transfer, Info> Data<T, Transfer, Info>
where
    Transfer: MessageTransfer,
    Info:  MessageInformation,
    T: DataHandler
{
    pub async fn add_user_request(
        &self,
        info: Info,
        validator: impl Fn(&Transfer) -> bool + Send + Sync + 'static,
        sender: OneShotSender<Transfer>,
    ) {
        let mut requests = self.pending_requests.lock().await;
        if let Some(reqs) = requests.get_mut(&info) {
            reqs.push((Box::new(validator), sender));
            return;
        }

        requests.insert(info, vec![(Box::new(validator), sender)]);
    }

    pub async fn get_request(
        &self,
        message: &Transfer,
    ) -> BinaryOptionsResult<Option<Vec<OneShotSender<Transfer>>>> {
        let mut requests = self.pending_requests.lock().await;
        let info = message.info();

        if let Some(reqs) = requests.get_mut(&info) {
            // Find the index of the matching validator
            let mut senders = Vec::new();
            let mut keepers = Vec::new();
            let drain = reqs.drain(std::ops::RangeFull);
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
                        .map(|(_, s)| s)
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
                    sender.send(error.clone())?;
                }
            }
        }
        Ok(None)
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
impl<T, Transfer, Info>  Data<T, Transfer, Info>
    where
        Transfer: MessageTransfer + 'static,
        Info: MessageInformation,
        T: DataHandler
{
    pub async fn update_data(&self, message: Transfer, sender: &Sender<Message>) -> BinaryOptionsResult<()>
    {
        if let Some(request) = message.user_request::<Transfer, Info>() {
            self.add_user_request(request.info, request.validator, request.sender).await;
            let message = *request.message;
            if let Err(e) = sender.send(message.into()).await {
                warn!("Error sending message: {}", BinaryOptionsToolsError::from(e));
            }

        } else {
            self.update(&message).await;
            if let Some(senders) = self.get_request(&message).await? {
                for s in senders {
                    s.send(message.clone())?;
                }
            }
        }
        Ok(())
    }

}

impl<Transfer, Info> UserRequest<Transfer, Info>
where 
    Transfer: MessageTransfer,
    Info: MessageInformation
{
    pub fn new(
        message: Transfer,
        info: Info,
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

impl<Transfer, Info> Clone for UserRequest<Transfer, Info>
where 
    Transfer: MessageTransfer + 'static,
    Info: MessageInformation
{
    fn clone(&self) -> Self {
        let (sender, _) = tokio::sync::oneshot::channel();
        Self {
            message: self.message.clone(),
            info: self.info.clone(),
            validator: Box::new(default_validator),
            sender
        }
    }
}

pub fn default_validator<Transfer: MessageTransfer>(_val: &Transfer) -> bool  
{
    false
}

impl<'de, Transfer, Info> Deserialize<'de> for UserRequest<Transfer, Info>
where 
    Transfer: MessageTransfer + 'static,
    Info: MessageInformation

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
        let info: Info = serde_json::from_value(
            value
                .get("info")
                .ok_or(serde::de::Error::missing_field(
                    "Missing field 'info'",
                ))?

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
