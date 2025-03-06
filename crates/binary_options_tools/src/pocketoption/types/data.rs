use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use async_channel::{Receiver, Sender, bounded};
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use binary_options_tools_core::{
    constants::MAX_CHANNEL_CAPACITY, error::BinaryOptionsResult, general::traits::DataHandler,
};

use crate::pocketoption::{
    error::PocketResult, parser::message::WebSocketMessage, ws::stream::StreamAsset,
};

use super::{
    order::Deal,
    update::{UpdateAssets, UpdateBalance, UpdateStream},
};

pub struct Channels(Sender<WebSocketMessage>, Receiver<WebSocketMessage>);

#[derive(Default, Clone)]
pub struct PocketData {
    balance: Arc<Mutex<UpdateBalance>>,
    opened_deals: Arc<Mutex<HashMap<Uuid, Deal>>>,
    closed_deals: Arc<Mutex<HashSet<Deal>>>,
    payout_data: Arc<Mutex<HashMap<String, i32>>>,
    server_time: Arc<Mutex<i64>>,
    stream_channels: Arc<Channels>,
    stream_assets: Arc<Mutex<Vec<String>>>,
}

impl Default for Channels {
    fn default() -> Self {
        let (s, r) = bounded(MAX_CHANNEL_CAPACITY);
        Self(s, r)
    }
}

impl From<UpdateAssets> for HashMap<String, i32> {
    fn from(value: UpdateAssets) -> Self {
        value
            .0
            .iter()
            .map(|a| (a.symbol.clone(), a.payout))
            .collect()
    }
}

impl PocketData {
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

    pub async fn clean_closed_deals(&self) {
        let mut closed = self.closed_deals.lock().await;
        closed.clear();
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

    pub async fn update_server_time(&self, time: i64) {
        let mut s_time = self.server_time.lock().await;
        *s_time = time;
    }

    pub async fn get_server_time(&self) -> i64 {
        // *self.server_time.lock().await
        (Utc::now() + Duration::from_secs(2 * 3600 + 123)).timestamp()
    }

    pub async fn add_stream(&self, asset: String) -> StreamAsset {
        info!("Created new channels and StreamAsset instance");
        let mut assets = self.stream_assets.lock().await;
        assets.push(asset.clone());
        StreamAsset::new(self.stream_channels.1.clone(), asset)
    }

    pub async fn add_stream_chuncked(&self, asset: String, chunck_size: usize) -> StreamAsset {
        info!("Created new channels and StreamAsset instance");
        let mut assets = self.stream_assets.lock().await;
        assets.push(asset.clone());
        StreamAsset::new_chuncked(self.stream_channels.1.clone(), asset, chunck_size)
    }

    pub async fn add_stream_timed(&self, asset: String, time: Duration) -> StreamAsset {
        info!("Created new channels and StreamAsset instance");
        let mut assets = self.stream_assets.lock().await;
        assets.push(asset.clone());
        StreamAsset::new_timed(self.stream_channels.1.clone(), asset, time)
    }

    pub async fn stream_assets(&self) -> Vec<String> {
        self.stream_assets.lock().await.clone()
    }

    pub async fn send_stream(&self, stream: UpdateStream) -> PocketResult<()> {
        if self.stream_channels.0.receiver_count() > 1 {
            self.stream_channels
                .0
                .force_send(WebSocketMessage::UpdateStream(stream))?;
        }
        Ok(())
    }
}

#[async_trait]
impl DataHandler for PocketData {
    type Transfer = WebSocketMessage;

    async fn update(&self, message: &WebSocketMessage) -> BinaryOptionsResult<()> {
        match message {
            WebSocketMessage::SuccessupdateBalance(balance) => {
                self.update_balance(balance.clone()).await
            }
            WebSocketMessage::UpdateAssets(assets) => {
                // let mut file: std::fs::File = OpenOptions::new().create(true).truncate(true).write(true).open("tests/assets2.txt").unwrap();
                // file.write_all(serde_json::to_string(assets).unwrap().as_bytes());
                self.update_payout_data(assets.clone()).await
            }
            WebSocketMessage::UpdateClosedDeals(deals) => {
                self.update_closed_deals(deals.0.clone()).await
            }
            WebSocketMessage::UpdateOpenedDeals(deals) => {
                self.update_opened_deals(deals.0.clone()).await
            }
            WebSocketMessage::SuccesscloseOrder(order) => {
                self.update_closed_deals(order.deals.clone()).await
            }
            WebSocketMessage::SuccessopenOrder(order) => {
                self.update_opened_deals(vec![order.clone()]).await
            }
            WebSocketMessage::UpdateStream(stream) => {
                match stream.0.first() {
                    Some(item) => self.update_server_time(item.time.timestamp()).await,
                    None => warn!("Missing data in 'updateStream' message"),
                }
                self.send_stream(stream.clone()).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

/*
    async fn update_loop(
        data: Data,
        reciever: &mut Receiver<WebSocketMessage>,
        sender: &Sender<Message>,
    ) -> PocketResult<()> {
        while let Some(msg) = reciever.recv().await {
            match msg {
                WebSocketMessage::SuccessupdateBalance(balance) => {
                    data.update_balance(balance).await
                }
                WebSocketMessage::UpdateAssets(assets) => data.update_payout_data(assets).await,
                WebSocketMessage::UpdateClosedDeals(deals) => {
                    data.update_closed_deals(deals.0).await
                }
                WebSocketMessage::UpdateOpenedDeals(deals) => {
                    data.update_opened_deals(deals.0).await
                }
                WebSocketMessage::SuccesscloseOrder(order) => {
                    data.update_closed_deals(order.deals).await
                }
                WebSocketMessage::SuccessopenOrder(order) => {
                    data.update_opened_deals(vec![order.into()]).await
                }
                WebSocketMessage::UserRequest(request) => {
                    data.add_user_request(request.info, request.validator, request.sender)
                        .await;
                    if request.message.info() == WebSocketMessage::None.info() {
                        continue;
                    }
                    if let Err(e) = sender.send(request.message.into()).await {
                        warn!("Error sending message: {}", PocketOptionError::from(e));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

*/
