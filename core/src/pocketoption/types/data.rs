use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_channel::{bounded, Receiver, Sender};
use tokio::sync::{oneshot::Sender as OneshotSender, Mutex};
use tracing::debug;
use uuid::Uuid;

use crate::{
    contstants::MAX_CHANNEL_CAPACITY,
    pocketoption::{
        error::PocketResult, parser::message::WebSocketMessage, ws::stream::StreamAsset,
    },
};

use super::{
    info::MessageInfo,
    order::Deal,
    update::{UpdateAssets, UpdateBalance, UpdateStream},
};
type Element = (
    Box<dyn Fn(&WebSocketMessage) -> bool + Send + Sync>,
    OneshotSender<WebSocketMessage>,
);
type HashMapData = HashMap<MessageInfo, Vec<Element>>;

pub struct Channels(Sender<WebSocketMessage>, Receiver<WebSocketMessage>);

#[derive(Default, Clone)]
pub struct Data {
    balance: Arc<Mutex<UpdateBalance>>,
    opened_deals: Arc<Mutex<HashMap<Uuid, Deal>>>,
    closed_deals: Arc<Mutex<HashSet<Deal>>>,
    payout_data: Arc<Mutex<HashMap<String, i32>>>,
    pending_requests: Arc<Mutex<HashMapData>>,
    server_time: Arc<Mutex<i64>>,
    stream_channels: Arc<Channels>,
}

impl Data {
    pub async fn update_balance(&self, balance: UpdateBalance) {
        let mut blnc = self.balance.lock().await;
        *blnc = balance;
    }

    pub async fn get_balance(&self) -> UpdateBalance {
        self.balance.lock().await.clone()
    }

    pub async fn update_opened_deals(&self, deals: impl Into<Vec<Deal>>) {
        let mut opened = self.opened_deals.lock().await;
        let new_deals: HashMap<Uuid, Deal> = HashMap::from_iter(
            deals
                .into()
                .into_iter()
                .map(|d| (d.id, d))
                .collect::<Vec<(Uuid, Deal)>>(),
        );
        opened.extend(new_deals);
    }

    pub async fn get_opened_deals(&self) -> Vec<Deal> {
        self.opened_deals
            .lock()
            .await
            .clone()
            .into_values()
            .collect()
    }

    async fn remove_opened_deal(&self, id: Uuid) {
        let mut opened = self.opened_deals.lock().await;
        opened.remove(&id);
    }

    pub async fn update_closed_deals(&self, deals: impl Into<Vec<Deal>>) {
        let mut closed = self.closed_deals.lock().await;
        let deals = deals.into();
        for d in deals.iter() {
            self.remove_opened_deal(d.id).await;
        }
        let new: HashSet<Deal> = HashSet::from_iter(deals);
        closed.extend(new);
    }

    pub async fn get_closed_deals(&self) -> Vec<Deal> {
        self.closed_deals.lock().await.clone().into_iter().collect()
    }

    pub async fn update_payout_data(&self, payout: UpdateAssets) {
        let mut data = self.payout_data.lock().await;
        *data = payout.into();
    }

    pub async fn get_full_payout(&self) -> HashMap<String, i32> {
        self.payout_data.lock().await.clone()
    }

    pub async fn get_payout(&self, asset: impl ToString) -> Option<i32> {
        self.payout_data
            .lock()
            .await
            .get(&asset.to_string())
            .cloned()
    }

    pub async fn add_user_request(
        &self,
        info: MessageInfo,
        validator: impl Fn(&WebSocketMessage) -> bool + Send + Sync + 'static,
        sender: tokio::sync::oneshot::Sender<WebSocketMessage>,
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
        message: &WebSocketMessage,
    ) -> PocketResult<Option<Vec<OneshotSender<WebSocketMessage>>>> {
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
                        .collect::<Vec<OneshotSender<WebSocketMessage>>>(),
                ));
            } else {
                return Ok(None);
            }
        }
        if let WebSocketMessage::FailOpenOrder(fail) = message {
            if let Some(reqs) = requests.remove(&MessageInfo::SuccessopenOrder) {
                for (_, sender) in reqs.into_iter() {
                    sender.send(WebSocketMessage::FailOpenOrder(fail.clone()))?;
                }
            }
        }
        Ok(None)
    }

    pub async fn update_server_time(&self, time: i64) {
        let mut s_time = self.server_time.lock().await;
        *s_time = time;
    }

    pub async fn get_server_time(&self) -> i64 {
        *self.server_time.lock().await
    }

    pub async fn add_stream(&self, asset: String) -> StreamAsset {
        debug!("Created new channels and StreamAsset instance");
        StreamAsset::new(self.stream_channels.1.clone(), asset)
    }

    pub async fn send_stream(&self, stream: UpdateStream) -> PocketResult<()> {
        if self.stream_channels.1.receiver_count() > 1 {
            return Ok(self
                .stream_channels
                .0
                .send(WebSocketMessage::UpdateStream(stream))
                .await?);
        }
        Ok(())
    }
}

impl Default for Channels {
    fn default() -> Self {
        let (s, r) = bounded(MAX_CHANNEL_CAPACITY);
        Self(s, r)
    }
}
