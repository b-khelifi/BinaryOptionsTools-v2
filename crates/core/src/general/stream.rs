use async_channel::Receiver;
use futures_util::{stream::unfold, Stream};

pub struct RecieverStream<T> {
    inner: Receiver<T>,
}

impl<T> RecieverStream<T> {
    pub fn new(inner: Receiver<T>) -> Self {
        Self { inner }
    }

    async fn receive(&self) -> anyhow::Result<T> {
        Ok(self.inner.recv().await?)
    }

    pub fn to_stream(&self) -> impl Stream<Item = anyhow::Result<T>> + '_ {
        Box::pin(unfold(self, |state| async move {
            let item = state.receive().await;
            Some((item, state))
        }))
    }
}
