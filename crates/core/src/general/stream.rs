use std::{sync::Arc, time::Duration};

use async_channel::Receiver;
use futures_util::{stream::unfold, Stream};

use crate::{error::BinaryOptionsResult, utils::time::timeout};

pub struct RecieverStream<T> {
    inner: Receiver<T>,
    timeout: Option<Duration>
}

impl<T> RecieverStream<T> {
    pub fn new(inner: Receiver<T>) -> Self {
        Self { inner, timeout: None }
    }

    pub fn new_timed(inner: Receiver<T>, timeout: Option<Duration>) -> Self {
        Self { inner, timeout }
    }

    async fn receive(&self) -> BinaryOptionsResult<T> {
        match self.timeout {
            Some(time) => timeout(time, self.inner.recv(), "RecieverStream".to_string()).await,
            None => Ok(self.inner.recv().await?)
        }
    }

    pub fn to_stream(&self) -> impl Stream<Item = BinaryOptionsResult<T>> + '_ {
        Box::pin(unfold(self, |state| async move {
            let item = state.receive().await;
            Some((item, state))
        }))
    }

    pub fn to_stream_static(self: Arc<Self>) -> impl Stream<Item =   BinaryOptionsResult<T>> + 'static 
    where T: 'static {
        Box::pin(unfold(self, |state| async move {
            let item = state.receive().await;
            Some((item, state))
        }))

    }  

}
